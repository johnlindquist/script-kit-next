try:
    import ujson as json
except ImportError:
    import json

try:
    import utime as time
except ImportError:
    import time

try:
    import config
except ImportError:
    config = None


TASKS_FILE = "tasks.md"
STATE_FILE = "state.json"
DEFAULT_DURATION_MS = 25 * 60 * 1000
DRAW_INTERVAL_MS = 1000

tasks = []
selected = 0
timer_running = False
timer_started_at = 0
timer_remaining_ms = DEFAULT_DURATION_MS
sessions_done = 0
completed_ids = {}
last_sync_message = "Ready"
last_error = ""
dirty = True
last_draw_at = -1
initialized = False


def _ticks_ms():
    if hasattr(time, "ticks_ms"):
        return time.ticks_ms()
    return int(time.time() * 1000)


def _ticks_diff(newer, older):
    if hasattr(time, "ticks_diff"):
        return time.ticks_diff(newer, older)
    return newer - older


def _screen_size():
    return int(getattr(screen, "width", 160)), int(getattr(screen, "height", 120))


def _set_pen(r, g, b):
    try:
        screen.pen = color.rgb(r, g, b)
        return
    except Exception:
        pass

    try:
        screen.brush = brushes.color(r, g, b)
    except Exception:
        pass


def _clear(r, g, b):
    _set_pen(r, g, b)
    try:
        screen.clear()
    except Exception:
        pass


def _rect(x, y, w, h, r, g, b):
    _set_pen(r, g, b)
    try:
        screen.rectangle(int(x), int(y), int(w), int(h))
        return
    except Exception:
        pass

    try:
        screen.draw(shapes.rectangle(int(x), int(y), int(w), int(h)))
    except Exception:
        pass


def _text(value, x, y, r=255, g=255, b=255):
    _set_pen(r, g, b)
    try:
        screen.text(str(value), int(x), int(y))
    except Exception:
        pass


def _measure(value):
    try:
        width, height = screen.measure_text(str(value))
        return int(width), int(height)
    except Exception:
        return len(str(value)) * 6, 8


def _wrap(value, max_width, max_lines):
    words = str(value).split()
    lines = []
    current = ""

    for word in words:
        test = word if not current else current + " " + word
        width, _ = _measure(test)
        if width <= max_width:
            current = test
        else:
            if current:
                lines.append(current)
            current = word
        if len(lines) >= max_lines:
            break

    if current and len(lines) < max_lines:
        lines.append(current)

    if len(lines) == max_lines and words:
        width, _ = _measure(lines[-1])
        if width > max_width:
            while lines[-1] and _measure(lines[-1] + "...")[0] > max_width:
                lines[-1] = lines[-1][:-1]
        if not lines[-1].endswith("...") and len(words) > len(" ".join(lines).split()):
            lines[-1] = lines[-1] + "..."

    return lines or [""]


def _button_const(name):
    const = globals().get("BUTTON_" + name)
    io_obj = globals().get("io")
    if const is None and io_obj is not None:
        const = getattr(io_obj, "BUTTON_" + name, None)
    return const


def _pressed(name):
    const = _button_const(name)
    badge_obj = globals().get("badge")

    if badge_obj is not None and const is not None:
        try:
            return bool(badge_obj.pressed(const))
        except Exception:
            pass

    io_obj = globals().get("io")
    if io_obj is not None and const is not None:
        try:
            return const in io_obj.pressed
        except Exception:
            pass

    return False


def _duration_ms(token):
    if not token.startswith("@"):
        return None

    value = token[1:]
    multiplier = 60 * 1000
    if value.endswith("h"):
        multiplier = 60 * 60 * 1000
        value = value[:-1]
    elif value.endswith("m"):
        value = value[:-1]
    elif value.endswith("s"):
        multiplier = 1000
        value = value[:-1]

    try:
        return max(1, int(value)) * multiplier
    except ValueError:
        return None


def _task_id(task):
    return task["title"] + "|" + task.get("due", "") + "|" + str(task.get("duration_ms", DEFAULT_DURATION_MS))


def _parse_task_line(line, section, order):
    stripped = line.strip()
    lower = stripped.lower()

    if lower.startswith("- [x]"):
        return None
    if not lower.startswith("- [ ]"):
        return None

    body = stripped[5:].strip()
    if body.startswith("]"):
        body = body[1:].strip()

    title_parts = []
    tags = []
    due = ""
    priority = 0
    duration = DEFAULT_DURATION_MS

    for token in body.split():
        token_lower = token.lower()
        parsed_duration = _duration_ms(token)

        if parsed_duration is not None:
            duration = parsed_duration
        elif token_lower.startswith("due:") or token_lower.startswith("at:"):
            due = token.split(":", 1)[1]
        elif token.startswith("#") and len(token) > 1:
            tags.append(token)
        elif token == "!":
            priority = 1
        else:
            title_parts.append(token)

    title = " ".join(title_parts).strip()
    if not title:
        return None

    task = {
        "title": title,
        "section": section,
        "tags": " ".join(tags),
        "due": due,
        "priority": priority,
        "duration_ms": duration,
        "order": order,
    }
    task["id"] = _task_id(task)
    return task


def _load_tasks_from_text(markdown):
    parsed = []
    section = "Tasks"
    order = 0

    for raw_line in markdown.split("\n"):
        line = raw_line.strip()
        if line.startswith("#"):
            section = line.lstrip("#").strip() or "Tasks"
            continue

        task = _parse_task_line(line, section, order)
        if task is not None and not completed_ids.get(task["id"], False):
            parsed.append(task)
            order += 1

    def sort_key(task):
        due = task.get("due") or "99:99"
        return (-task.get("priority", 0), due, task.get("order", 0))

    parsed.sort(key=sort_key)
    return parsed


def load_tasks():
    global tasks, selected, timer_remaining_ms, last_sync_message, last_error

    try:
        with open(TASKS_FILE, "r") as task_file:
            markdown = task_file.read()
        tasks = _load_tasks_from_text(markdown)
        selected = min(selected, max(0, len(tasks) - 1))
        if tasks:
            timer_remaining_ms = tasks[selected]["duration_ms"]
        last_sync_message = str(len(tasks)) + " active"
        last_error = ""
    except Exception as exc:
        tasks = []
        selected = 0
        last_error = "Load failed"
        last_sync_message = str(exc)


def load_state():
    global sessions_done, completed_ids, selected, timer_remaining_ms

    try:
        with open(STATE_FILE, "r") as state_file:
            state = json.loads(state_file.read())
        sessions_done = int(state.get("sessions_done", 0))
        completed_ids = state.get("completed_ids", {})
        selected = int(state.get("selected", 0))
        timer_remaining_ms = int(state.get("timer_remaining_ms", DEFAULT_DURATION_MS))
    except Exception:
        sessions_done = 0
        completed_ids = {}


def save_state():
    state = {
        "sessions_done": sessions_done,
        "completed_ids": completed_ids,
        "selected": selected,
        "timer_remaining_ms": timer_remaining_ms,
    }
    try:
        with open(STATE_FILE, "w") as state_file:
            state_file.write(json.dumps(state))
    except Exception:
        pass


def _raw_url():
    if config is None:
        return ""
    return getattr(config, "GIST_RAW_URL", "")


def _wifi_value(name):
    if config is not None:
        value = getattr(config, name, "")
        if value:
            return value

    try:
        import secrets
        for candidate in (name, name.replace("WIFI_", "")):
            value = getattr(secrets, candidate, "")
            if value:
                return value
    except Exception:
        pass

    return ""


def _connect_wifi():
    try:
        import network
    except Exception:
        return True

    try:
        wlan = network.WLAN(network.STA_IF)
        wlan.active(True)
        if wlan.isconnected():
            return True

        ssid = _wifi_value("WIFI_SSID")
        password = _wifi_value("WIFI_PASSWORD")
        if not ssid:
            return False

        wlan.connect(ssid, password)
        deadline = _ticks_ms() + (getattr(config, "SYNC_TIMEOUT_SECONDS", 12) * 1000 if config else 12000)
        while not wlan.isconnected() and _ticks_diff(deadline, _ticks_ms()) > 0:
            try:
                time.sleep_ms(250)
            except Exception:
                time.sleep(0.25)
        return wlan.isconnected()
    except Exception:
        return False


def sync_from_gist():
    global last_sync_message, last_error, dirty

    url = _raw_url()
    if not url:
        last_error = "Set GIST_RAW_URL"
        last_sync_message = "No sync URL"
        dirty = True
        return

    if not _connect_wifi():
        last_error = "WiFi offline"
        last_sync_message = "Sync failed"
        dirty = True
        return

    try:
        try:
            import urequests as requests
        except ImportError:
            import requests

        response = requests.get(url)
        markdown = response.text
        try:
            response.close()
        except Exception:
            pass

        if not markdown or "- [ ]" not in markdown:
            last_error = "No tasks found"
            last_sync_message = "Sync ignored"
            dirty = True
            return

        with open(TASKS_FILE, "w") as task_file:
            task_file.write(markdown)
        load_tasks()
        last_sync_message = "Synced " + str(len(tasks))
        last_error = ""
    except Exception as exc:
        last_error = "Sync failed"
        last_sync_message = str(exc)

    dirty = True


def _current_task():
    if not tasks:
        return None
    return tasks[max(0, min(selected, len(tasks) - 1))]


def _format_time(ms):
    seconds = max(0, int(ms / 1000))
    minutes = int(seconds / 60)
    seconds = seconds % 60
    return "{:02d}:{:02d}".format(minutes, seconds)


def _timer_remaining():
    if not timer_running:
        return timer_remaining_ms
    elapsed = _ticks_diff(_ticks_ms(), timer_started_at)
    return max(0, timer_remaining_ms - elapsed)


def _select(delta):
    global selected, timer_running, timer_remaining_ms, dirty

    if not tasks:
        return

    selected = (selected + delta) % len(tasks)
    timer_running = False
    timer_remaining_ms = tasks[selected]["duration_ms"]
    dirty = True
    save_state()


def _toggle_timer():
    global timer_running, timer_started_at, timer_remaining_ms, dirty

    task = _current_task()
    if task is None:
        return

    if timer_running:
        timer_remaining_ms = _timer_remaining()
        timer_running = False
    else:
        if timer_remaining_ms <= 0:
            timer_remaining_ms = task["duration_ms"]
        timer_started_at = _ticks_ms()
        timer_running = True

    dirty = True
    save_state()


def _complete_timer_session():
    global sessions_done, timer_running, timer_remaining_ms, dirty

    task = _current_task()
    if task is None:
        return

    sessions_done += 1
    timer_running = False
    timer_remaining_ms = task["duration_ms"]
    dirty = True
    save_state()


def _mark_done():
    global selected, timer_running, timer_remaining_ms, dirty, last_sync_message

    task = _current_task()
    if task is None:
        return

    completed_ids[task["id"]] = True
    last_sync_message = "Done: " + task["title"][:18]
    timer_running = False
    load_tasks()
    if tasks:
        selected = min(selected, len(tasks) - 1)
        timer_remaining_ms = tasks[selected]["duration_ms"]
    dirty = True
    save_state()


def handle_buttons():
    if _pressed("UP"):
        _select(-1)
    elif _pressed("DOWN"):
        _select(1)

    if _pressed("A"):
        _toggle_timer()
    if _pressed("B"):
        _mark_done()
    if _pressed("C"):
        sync_from_gist()


def draw_empty(width, height):
    _clear(18, 22, 28)
    _text("Focus Badge", 8, 8, 255, 255, 255)
    _text("No active tasks", 8, 34, 255, 210, 80)
    _text("Edit tasks.md", 8, 54, 180, 190, 200)
    _text("C syncs Gist", 8, 72, 180, 190, 200)
    if last_error:
        _text(last_error, 8, height - 18, 255, 100, 100)


def draw_focus():
    width, height = _screen_size()
    task = _current_task()

    if task is None:
        draw_empty(width, height)
        return

    remaining = _timer_remaining()
    total = max(1, task["duration_ms"])
    progress = max(0, min(width - 16, int((width - 16) * (total - remaining) / total)))

    _clear(12, 16, 22)
    _rect(0, 0, width, 20, 32, 40, 52)
    _text("Focus", 8, 5, 255, 255, 255)
    _text(str(selected + 1) + "/" + str(len(tasks)), width - 42, 5, 190, 205, 220)

    y = 28
    if task.get("due"):
        _text(task["due"], 8, y, 255, 210, 90)
        y += 14

    for line in _wrap(task["title"], width - 16, 3):
        _text(line, 8, y, 255, 255, 255)
        y += 14

    if task.get("tags"):
        _text(task["tags"], 8, y + 2, 130, 210, 255)

    clock_y = max(72, height - 42)
    _text(_format_time(remaining), 8, clock_y, 255, 255, 255)
    _text("Run" if timer_running else "Pause", width - 48, clock_y, 170, 190, 210)

    bar_y = height - 18
    _rect(8, bar_y, width - 16, 6, 42, 52, 66)
    _rect(8, bar_y, progress, 6, 75, 220, 120)

    footer = "A start  B done  C sync"
    if width < 200:
        footer = "A start B done C sync"
    _text(footer, 8, height - 10, 140, 150, 160)

    if last_error:
        _text(last_error[:24], 8, height - 28, 255, 100, 100)
    elif last_sync_message:
        _text(last_sync_message[:24], 8, height - 28, 140, 170, 190)


def init():
    global initialized, dirty

    load_state()
    load_tasks()
    initialized = True
    dirty = True


def update():
    global dirty, last_draw_at, timer_running, timer_remaining_ms

    if not initialized:
        init()

    handle_buttons()

    if timer_running and _timer_remaining() <= 0:
        timer_remaining_ms = 0
        timer_running = False
        _complete_timer_session()

    now = _ticks_ms()
    should_draw = dirty or last_draw_at < 0 or _ticks_diff(now, last_draw_at) >= DRAW_INTERVAL_MS
    if should_draw:
        draw_focus()
        last_draw_at = now
        dirty = False


def on_exit():
    save_state()


run(update)
