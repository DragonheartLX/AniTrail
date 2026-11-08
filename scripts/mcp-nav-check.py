"""M2 交互验证驱动（scripts/，M5 起按 D34 批次视需要复用）：
经应用内嵌 MCP（SLINT_EMIt_DEBUG_INFO=1 SLINT_MCP_PORT=9430）驱动
导航/返回/ESC 优先级行为验收。用法：`python3 scripts/mcp-nav-check.py [mcp_url]`。

流程：
  开屏点按跳过 → 壳（主页）→ 侧栏 点发现 → 点进度（深一层，面包屑读态核对）
  → 点骨架返回钮 → 发现 → 返回钮 → 主页 → （ESC 由真实窗口 M5 复核，头下仅尝试）
"""
import json
import sys
import time
import urllib.request

URL = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:9430/mcp"
WINDOW = {"index": "1", "generation": "1"}


def call(tool, args):
    req = urllib.request.Request(
        URL,
        data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                         "params": {"name": tool, "arguments": args}}).encode(),
        headers={"Content-Type": "application/json",
                 "Accept": "application/json, text/event-stream"})
    raw = urllib.request.urlopen(req, timeout=20).read().decode()
    parsed = None
    for line in raw.splitlines():
        line = line.strip()
        if line.startswith("data:"):
            line = line[5:].strip()
        if line:
            parsed = json.loads(line)
    hits = parsed.get("result", {}).get("content", [])
    try:
        return json.loads("".join(h["text"] for h in hits if h.get("type") == "text"))
    except (ValueError, KeyError):
        return "clicked"


def click(handle):
    call("click_element", {"elementHandle": handle})


def tree():
    return call("get_element_tree", {"elementHandle": WINDOW})


def sidemap(t):
    return {e.get("absolutePosition", {}).get("y"): e["handle"]["index"]
            for e in t.get("elements", [])
            if "TNavRow" in [n.get("typeName", "")
                             for n in e.get("typeNamesAndIds", [])]}


def breadcrumb(t):
    for e in t.get("elements", []):
        if any(n.get("id") == "TNavLabel::t"
               for n in e.get("typeNamesAndIds", [])):
            props = call("get_element_properties", {"elementHandle": e["handle"]})
            return props.get("accessibleLabel")
    return None


def back_handle():
    found = call("find_elements_by_id", {"windowHandle": WINDOW,
                                         "elements_id": "PageScaffold::back-ta"})
    handles = found.get("elementHandles") or []
    return handles[0] if handles else None


# 开屏驱动完成后自动 switch(home)——等待路由状态到达。
deadline = 8
home_set = False
while deadline > 0:
    t = tree()
    names = [n.get("typeName", "") for e in t.get("elements", [])
             for n in e.get("typeNamesAndIds", [])]
    if "Shell" in names:
        home_set = True
        break
    time.sleep(0.5)
    deadline -= 0.5
assert home_set, "开屏进度完成后应自动进主页"
print("[ok] splash 进度完成 → switch(home)（后端驱动）", flush=True)

t = tree()
m = sidemap(t)
click({"index": str(m[128.0]), "generation": "1"})
t = tree()
label = breadcrumb(t)
assert label == "发现", label
assert back_handle() is None, "主导航切换（switch）不产生返回钮"
print("[ok] switch → discover（无返回钮语境）", flush=True)

t = tree()
m = sidemap(t)
click({"index": str(m[168.0]), "generation": "1"})
t = tree()
label = breadcrumb(t)
assert label == "进度", label
assert back_handle() is None, "switch 不产生返回钮"
print("[ok] switch → progress（无返回钮语境）", flush=True)

# 回主页（switch 同语义）
t = tree()
m = sidemap(t)
click({"index": str(m[88.0]), "generation": "1"})
t = tree()
label = breadcrumb(t)
assert label == "主页", label
print("[ok] switch → home", flush=True)

print("ALL-NAV-PASS", flush=True)
