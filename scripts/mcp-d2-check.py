"""D2 验收驱动（scripts/，M2 完成用）：下钻→滚动→返回→重入→回滚校验。

前提（先单独跑）：`SLINT_BACKEND=headless SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9430 \
cargo run -q -p ani-trail --features slint/mcp`（应用常驻出 MCP 端点）。
探针（probe.slint / pages/probe-detail.slint / home 钻取钮）在 M5 起移除。

流程：
  等开屏真实进度完成（自动 switch→home）→ 点下钻（go→push，探针详情页 24 行可滚）
  → 滚轮下滚（取 content-top 变化）→ 点返回钮（返回主页）→ 再次下钻
  → 截图比页：重入回到保存的滚动位（≈上一步 content-top）→ 主页无返回钮断言。
"""
import base64
import json
import struct
import sys
import time
import urllib.request
import zlib

URL = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:9430/mcp"
WINDOW = {"index": "1", "generation": "1"}
BG_PAGE = {0x15, 0x18, 0x1D}


def call(tool, args):
    req = urllib.request.Request(
        URL, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                              "params": {"name": tool, "arguments": args}}).encode(),
        headers={"Content-Type": "application/json",
                 "Accept": "application/json, text/event-stream"})
    events = urllib.request.urlopen(req, timeout=30).read().decode()
    parsed_list = []
    for line in events.splitlines():
        line = line.strip()
        if line.startswith("data:"):
            line = line[5:].strip()
        if line:
            parsed_list.append(json.loads(line))
    # SSE 可能分帧：优先带 image 的响应，否则最后的文本响应。
    picked = None
    with_image = False
    for cand in parsed_list:
        hits = cand.get("result", {}).get("content") or []
        if any(h.get("type") == "image" for h in hits):
            if not with_image:
                picked = cand
                with_image = True
        elif not with_image:
            picked = cand
    hits = (picked or {}).get("result", {}).get("content") or []
    joined = "".join(h.get("text", "") for h in hits if h.get("type") == "text")
    try:
        return json.loads(joined)
    except ValueError:
        return "clicked"


def click(handle):
    print("click:", call("click_element", {"elementHandle": handle}), flush=True)


def find_id(label):
    out = call("find_elements_by_id",
               {"windowHandle": WINDOW, "elements_id": label})
    handles = out.get("elementHandles") or []
    return handles[0] if handles else None


def wait_item(label, timeout=20.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        found = call("find_elements_by_id", {"windowHandle": WINDOW,
                                             "elements_id": label})
        handles = found.get("elementHandles") or []
        if handles:
            return handles[0]
        time.sleep(0.4)
    raise AssertionError(f"未等到元素 {label}")


def page_names():
    tree = call("get_element_tree",
                {"elementHandle": WINDOW, "maxElements": 400})
    return [n.get("typeName", "")
            for e in tree.get("elements", [])
            for n in e.get("typeNamesAndIds", [])]


def screenshot_rows():
    data_b64 = None
    events = urllib.request.urlopen(
        urllib.request.Request(
            URL, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                                  "params": {"name": "take_screenshot",
                                             "arguments": {"windowHandle": WINDOW}}}).encode(),
            headers={"Content-Type": "application/json",
                     "Accept": "application/json, text/event-stream"}),
        timeout=30).read().decode()
    for line in events.splitlines():
        line = line.strip()
        if line.startswith("data:"):
            line = line[5:].strip()
        if not line:
            continue
        cand = json.loads(line)
        hits = cand.get("result", {}).get("content") or []
        data_b64 = "".join(h.get("data", "") for h in hits if h.get("type") == "image")
    png = base64.b64decode(data_b64)
    pos = 8
    idat = b""
    width = height = channels = None
    while pos < len(png):
        length, typ = struct.unpack(">I4s", png[pos:pos + 8])
        chunk = png[pos + 8:pos + 8 + length]
        pos += 12 + length
        if typ == b"IHDR":
            width, height, _, color_type = struct.unpack(">IIBB", chunk[:10])
            channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color_type]
        elif typ == b"IDAT":
            idat += chunk
        elif typ == b"IEND":
            break
    stride = width * channels
    raw = zlib.decompress(idat)
    rows = []
    prev = bytearray(stride)
    off = 0
    for _ in range(height):
        kind = raw[off]
        off += 1
        line = bytearray(raw[off:off + stride])
        off += stride
        if kind == 1:
            for i in range(channels, stride):
                line[i] = (line[i] + line[i - channels]) & 0xFF
        elif kind == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif kind == 3:
            for i in range(stride):
                left = line[i - channels] if i >= channels else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 0xFF
        elif kind == 4:
            for i in range(stride):
                left = line[i - channels] if i >= channels else 0
                up = prev[i]
                upleft = prev[i - channels] if i >= channels else 0
                p = left + up - upleft
                pa, pb, pc = abs(p - left), abs(p - up), abs(p - upleft)
                predictor = left if (pa <= pb and pa <= pc) else (up if pb <= pc else upleft)
                line[i] = (line[i] + predictor) & 0xFF
        rows.append(bytes(line))
    return channels, rows


def content_top(channels, rows):
    """内容区第一处非页面底色内容的 y（滚动后往下移出顶部，值应为 140 或顶行顶）。"""
    for y in range(140, 900, 2):
        px = rows[y][840 * channels:(840 * channels) + 3]
        if tuple(px) not in (tuple(BG_PAGE),) and sum(px) > 120:
            return y
    return None


# —— 1. 等开屏后端进度完成（自动 switch→home）——
deadline = time.time() + 20
while time.time() < deadline:
    names = page_names()
    if "Shell" in names:
        break
    time.sleep(0.4)
else:
    raise AssertionError("开屏进度应自动进主页")
print("[ok] splash 后端进度完成 → switch(home)", flush=True)

# —— 2. 等主页下钻钮就绪 ——
drill = wait_item("HomePage::drill-btn", 10.0)
print("[ok] home 就绪（下钻钮）", flush=True)

# —— 3. 下钻（go→push）。探针详情页 24 行可滚 ——
click(drill)
detail = wait_item("PageScaffold::back-ta", 10.0)
print("[ok] go(commit) → 探针详情页（返回钮在）", flush=True)

channels, rows = screenshot_rows()
top_before = content_top(channels, rows)

# —— 4. 滚动（分派两次滚轮）——
fl = [e for e in call("get_element_tree",
                      {"elementHandle": WINDOW, "maxElements": 400}).get("elements", [])
      if "Flickable" in [n.get("typeName") for n in e.get("typeNamesAndIds", [])]]
tall = [e for e in fl if (e.get("size") or {}).get("height", 0) > 200][0]
call("dispatch_pointer_scroll", {"windowHandle": WINDOW, "position": {"x": 800.0, "y": 500.0}, "deltaY": -400.0})
time.sleep(0.6)
channels, rows = screenshot_rows()
top_scrolled = content_top(channels, rows)
assert top_scrolled != top_before, f"滚动未生效：{top_before} → {top_scrolled}"
print(f"[ok] 滚动：content-top {top_before} → {top_scrolled}", flush=True)

# —— 5. 返回（pop→主页）——
call("click_element", {"elementHandle": detail})
drill = wait_item("HomePage::drill-btn", 10.0)
print("[ok] 返回钮 → 主页", flush=True)

# —— 6. 重入：应回滚到保存位（D2：重建后状态不丢）——
click(drill)
wait_item("PageScaffold::back-ta", 10.0)
channels, rows = screenshot_rows()
top_restored = content_top(channels, rows)
assert top_restored == top_scrolled, f"重入回滚不符：{top_restored} ≠ {top_scrolled}"
print(f"[ok] 重入回滚至保存位：content-top == {top_restored}", flush=True)

# —— 7. 返回主页：主页无返回钮（switch 语境）——
call("click_element", {"elementHandle": wait_item("PageScaffold::back-ta", 10.0)})
drill = wait_item("HomePage::drill-btn", 10.0)
assert find_id("PageScaffold::back-ta") is None, "主页不应有返回钮"
print("[ok] 主页无返回钮（switch 语境）", flush=True)

print("ALL-D2-PASS", flush=True)
