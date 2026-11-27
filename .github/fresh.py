"""Temporary: first visits to the About page, in fresh sessions of Safari."""

import glob
import json
import os
import subprocess
import sys
import time
import urllib.request

SITE = "http://127.0.0.1:1313"
SPY = """
window.log = [];
const mark = (what) => log.push(`${what}@${performance.now() | 0}`);
onerror = (it) => mark(it), onunhandledrejection = (it) => mark(it.reason);
const decode = HTMLImageElement.prototype.decode, context = HTMLCanvasElement.prototype.getContext;
HTMLImageElement.prototype.decode = function () {
  const name = this.src.split('/').pop().split('.')[0], done = decode.call(this);
  mark(`?${name}`), done.then(() => mark(`+${name}`), (it) => mark(`!${name} ${it} ${this.naturalWidth}`));
  return done;
};
HTMLCanvasElement.prototype.getContext = function (kind, ...rest) {
  if (kind !== 'webgl2') return context.call(this, kind, ...rest);
  for (const name of ['lost', 'restored', 'creationerror']) this.addEventListener('webglcontext' + name, (it) => mark(name + it.statusMessage));
  mark('?gl');
  const made = context.call(this, kind, ...rest);
  mark(`gl ${!!made} ${made?.isContextLost()}`);
  return made;
};
"""
STATE = "return [document.querySelector('.bed')?.className ?? 'none', performance.now() | 0, ...(window.log ?? [])].join(' ');"
PATHS = ["/posts/home/", "/posts/notes-on-euclidean-spaces/", "/about/"]
PLAIN = "<!doctype html><body style='background: #cde; font-size: 40px'>" + "<p>Some plain text, on a plain page.</p>" * 40
SPIN = """<canvas style="position: absolute; top: 0" width=400 height=900></canvas><script>
const pen = document.querySelector('canvas').getContext('2d');
(function next(now) { pen.clearRect(0, 0, 400, 900); pen.beginPath(); pen.arc(200, 200, 150, 0, now / 500 % 6); pen.stroke(); requestAnimationFrame(next); })(0);
</script>"""
way = sys.argv[1]


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    request = urllib.request.Request("http://127.0.0.1:4444" + path, data, {"Content-Type": "application/json"}, method=method)
    try:
        return json.load(urllib.request.urlopen(request))["value"]
    except urllib.error.HTTPError as error:
        return error.read().decode()[:400]


[script], [sheet] = glob.glob("result/assets/site.*.js"), glob.glob("result/assets/site.*.css")
source = open(script).read().replace("https://utteranc.es", SITE).replace("motion: reduce", "motion: never")
if way == "master":
    source = source.replace(".then(() => tide(element))", "")
open(script, "w").write("{" + SPY + "}\n" + ("" if way == "mute" else source))
for path in PATHS:
    page = open(f"result{path}index.html").read()
    if way != "branch":
        page = {"plain": PLAIN, "spin": PLAIN + SPIN}.get(way, page.replace('<canvas class="bed"></canvas>', ""))
    open(f"result{path}index.html", "w").write(page)
rules = open(sheet).read().replace("(prefers-reduced-motion: no-preference)", "screen")
open(sheet, "w").write(rules)
os.makedirs("shots", exist_ok=True)
driver = subprocess.Popen(["safaridriver", "--port=4444"], stdout=subprocess.DEVNULL)
server = subprocess.Popen([sys.executable, "-m", "http.server", "1313", "-b", "127.0.0.1", "-d", "result"], stderr=subprocess.DEVNULL)
for _ in range(60):
    try:
        urllib.request.urlopen(SITE, timeout=3)
        break
    except OSError:
        time.sleep(1)
try:
    for n in range(12):
        session = "/session/" + call("POST", "/session", {"capabilities": {"alwaysMatch": {}}})["sessionId"]
        run = lambda script: call("POST", session + "/execute/sync", {"script": script, "args": []})
        call("POST", session + "/window/rect", {"x": 0, "y": 0, "width": 1400, "height": 900})
        for path in PATHS:
            call("POST", session + "/url", {"url": SITE + path}), time.sleep(3)
            if n < 5 and "notes" in path:
                for pid in subprocess.run(["pgrep", "-f", "WebKit.(GPU|WebContent)"], capture_output=True, text=True).stdout.split():
                    subprocess.run(f"sudo sample {pid} 1 20 > shots/sample-{way}-{n}-{pid}.txt 2>&1", shell=True)
            print("SEEN", n, path, run(STATE), flush=True)
        if n < 1:
            subprocess.run(["screencapture", "-x", f"shots/screen-{way}.png"])
        call("DELETE", session)
    subprocess.run(["open", "-a", "Safari", SITE + "/about/"]), time.sleep(12)
    subprocess.run(["screencapture", "-x", f"shots/free-{way}.png"])
    words = "unresponsive|timeout|timed out|crash|terminat|iosurface|metal|gpuprocess|fault|error"
    subprocess.run(f"log show --last 4m --style compact --predicate 'process CONTAINS \"WebKit\" OR process == \"Safari\"' | grep -iE '{words}' | tail -1500 > shots/log-{way}.txt", shell=True)
    subprocess.run(f"(sw_vers; system_profiler SPDisplaysDataType; ls ~/Library/Logs/DiagnosticReports /Library/Logs/DiagnosticReports) > shots/mac-{way}.txt", shell=True)
finally:
    driver.kill(), server.kill()
