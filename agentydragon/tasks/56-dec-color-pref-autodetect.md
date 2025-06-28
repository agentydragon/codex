+++
id = "56"
title = "Support DEC color-preference autodetection"
status = "open"
freeform_status = ""
dependencies = []
last_updated = "2024-06-11T00:00:00Z"
+++

# Task: Support DEC color-preference autodetection

> _This task is specific to codex-rs._

## Acceptance Criteria

1. The TUI must listen for the DEC color-preference OSC sequence (`ESC [ ? 1002 l` / `ESC [ ? 1002 h`).
2. On startup (or when switching panes), if the terminal reports its preferred color mode (e.g. 8‑bit vs 24‑bit), the TUI respects that preference automatically.
3. Users can still override color mode manually via `tui.colors` config.
4. All existing color-themed widgets render correctly under both auto-detected and manually overridden modes.

## Implementation

**How it was implemented**

(To be completed by the developer.)

**How it works**

(To be completed by the developer.)

## Notes

- The DEC private mode for color-preference is defined in the VT520 manual.
- This may require enabling/reading terminal responses via the input loop.

## References

<https://github.com/contour-terminal/contour/blob/master/docs/vt-extensions/color-palette-update-notifications.md>

```markdown
# Dark and Light Mode detection

Most modern operating systems and desktop environments do support Dark and Light themes,
this includes at least MacOS, Windows, KDE Plasma, Gnome, and probably others.

Some even support switching from dark to light and light to dark mode based on sun rise / sun set.

In order to not make the terminal emulator look bad after such switch, we must
enable the applications **inside** the terminal to detect when the terminal has
updated the color palette. This may happen either due to the operating system having
changed the current theme or simply because the user has explicitly requested to
reconfigure the currently used theme (e.g. because the user requested to change the terminal profile,
also containing a different color scheme).

Ideally we are getting CLI tools like [delta]() to query the theme mode before sending out RGB values
to the terminal to make the output look more in line with the rest of the desktop.

But also TUIs like vim should be able to reflect dark/light mode changes as soon as the
desktop has changed to light/dark mode or the user has changed the terminal profile.

## Query the current theme mode?

Send `CSI ? 996 n` to the terminal to explicitly request the current
color preference (dark mode or light mode) by the operating system.

The terminal will reply back in either of the two ways:

VT sequence       | description
------------------|---------------------------------
`CSI ? 997 ; 1 n` | DSR reply to indicate dark mode
`CSI ? 997 ; 2 n` | DSR reply to indicate light mode

## Request unsolicited DSR on color palette updates

Send `CSI ? 2031 h` to the terminal to enable unsolicited DSR (device status report) messages
for color palette updates and `CSI ? 2031 l` respectively to disable it again.

The sent out DSR looks equivalent to the already above mentioned.
This notification is not just sent when dark/light mode has been changed
by the operating system / desktop, but also if the user explicitly changed color scheme,
e.g. by configuration.

### When to send out the DSR?

A terminal emulator should only send out the DSR when the palette has been updated due to a change in the
terminal emulator's color palette, e.g. because the user has changed the terminal profile directly
or indirectly by changing the operating system theme.

## Example source code

Please have a look at our example C++ [source code](https://github.com/contour-terminal/contour/blob/master/examples/detect-dark-light-mode.cpp)
in order to see how to implement this in your own application.

<... snip ...>

## Tools

* [rod](https://github.com/leiserfg/rod): Terminal Dark/Light Mode Detection Tool
```

<https://github.com/contour-terminal/contour/blob/master/examples/detect-dark-light-mode.cpp>:

```cpp
// SPDX-License-Identifier: Apache-2.0
#include <csignal>
#include <cstdlib>
#include <iostream>
#include <string_view>

#include <termios.h>
#include <unistd.h>

using namespace std::literals;

static bool processEvent(std::string_view response)
{
    if (response == "\033[?997;1n")
    {
        std::cout << "dark\n";
        return true;
    }
    else if (response == "\033[?997;2n")
    {
        std::cout << "light\n";
        return true;
    }

    std::cout << "unknown\n";
    return false;
}

static bool queryDarkLightModeOnce()
{
    // Also send DA1 to detect end of reply, in case the terminal does not support color mode detection
    std::cout << "\033[?996n\033[c";
    std::cout.flush();

    char buf[32];
    size_t n = 0;
    size_t i = 0;
    while (i < sizeof(buf))
    {
        if (read(STDIN_FILENO, buf + i, 1) != 1)
            break;
        else if (buf[i] == 'n')
            n = i + 1;
        else if (buf[i] == 'c')
        {
            ++i;
            break;
        }
        ++i;
    }

    return processEvent(std::string_view(buf, n));
}

static void signalHandler(int signo)
{
    std::signal(signo, SIG_DFL);
    std::cerr << "Received signal " << signo << ", exiting...\n";
}

static void monitorDarkLightModeChanges()
{
    char buf[32];
    size_t n = 0;
    size_t i = 0;
    while (true)
    {
        if (i >= sizeof(buf))
            i = 0;
        if (read(STDIN_FILENO, buf + i, 1) != 1)
            break;
        else if (buf[i] == '\033')
        {
            buf[0] = '\033';
            i = 1;
        }
        else if (buf[i] == 'n')
        {
            n = i + 1;
            auto const response = std::string_view(buf, n);
            processEvent(response);
        }
        ++i;
    }
}

int main(int argc, char* argv[])
{
    if (!isatty(STDIN_FILENO))
    {
        std::cerr << "stdin is not a terminal\n";
        return EXIT_FAILURE;
    }

    termios oldt {};
    termios newt {};
    tcgetattr(STDIN_FILENO, &oldt);
    newt = oldt;
    newt.c_lflag &= ~(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &newt);

    if (argc == 2 && (argv[1] == "-h"sv || argv[1] == "--help"sv))
    {
        std::cout << "Usage: " << argv[0] << " [monitor]\n";
        return EXIT_SUCCESS;
    }

    if (!queryDarkLightModeOnce())
        return EXIT_FAILURE;

    if (argc == 2 && argv[1] == "monitor"sv)
    {
        struct sigaction sa;
        sa.sa_handler = signalHandler;
        sa.sa_flags = 0; // Explicitly don't set SA_RESTART;
        sigemptyset(&sa.sa_mask);
        for (int const sig: { SIGTERM, SIGINT, SIGQUIT })
            sigaction(sig, &sa, nullptr);

        std::cerr << "Monitoring dark/light mode changes, press Ctrl+C to exit...\n";
        std::cerr << "\033[?2031h"; // Enable dark/light mode change notifications
        std::cerr.flush();
        monitorDarkLightModeChanges();
        std::cerr << "\033[?2031l"; // Disable dark/light mode change notifications
        std::cerr << "Finished monitoring dark/light mode changes\n";
        std::cerr.flush();
    }

    // restore terminal
    tcsetattr(STDIN_FILENO, TCSANOW, &oldt);

    return EXIT_SUCCESS;
}
```

<https://gist.github.com/ntrrgc/fc47b416bff68ebc05883ec733c8a7b8> :

```python
import os, termios, collections, re

TermAttr = collections.namedtuple("TermAttr",
    ["iflag", "oflag", "cflag", "lflag", "ispeed", "ospeed", "cc"])

old = TermAttr._make(termios.tcgetattr(0))
new = old._replace(
    lflag=old.lflag & ~(termios.ECHO | termios.ICANON)
)
try:
    termios.tcsetattr(0, termios.TCSADRAIN, list(new))
    REQUEST_FG = b"\x1b]10;?\x1b\\"
    REQUEST_BG = b"\x1b]11;?\x1b\\"
    os.write(1, REQUEST_FG + REQUEST_BG)
    re_rgb = re.compile(rb".*\x1b](10|11);rgb:(\w+)/(\w+)/(\w+)(?:\x1b\\|\x07)")
    matches = []
    response = b""
    while True:
        buf = os.read(0, 1)
        if buf == b"":
            break # EOF
        response += buf
        if match := re_rgb.match(response):
            matches.append(match)
            response = b""
            if len(matches) == 2:
                break
except KeyboardInterrupt:
    pass
finally:
    termios.tcsetattr(0, termios.TCSADRAIN, list(old))
for match in matches:
    feature, r, g, b = [x.decode() for x in match.groups()]
    feature = {"10": "Foreground", "11": "Background"}[feature]
    print(f"{feature} R={r} G={g} B={b}")
```
