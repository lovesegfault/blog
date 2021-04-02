+++
title = "macOS: The Good, The Bad, and The Ugly"
date = 2021-04-01T22:10:05-07:00
tags = [ "macos", "linux", "macbook", "opinion" ]
categories = [ "essay" ]
draft = true
+++

## Introduction

As far as people's usual interaction with computers go, I think I've had a
fairly unusual story, even for someone who works in the industry.
I grew up in Brazil, where electronics are particularly expensive, and to add
insult to injury one of the few things my parents seemed to not argue about was
how much they hated computers. Unsurprisingly, then, I grew up with a desktop
that was all-around garbage and running Windows on it, from XP to 7, was a
painful endeavor.

{{< figure src="./images/ubuntu-cd.jpg" caption="Back in the day!">}}

Luckily, when I was around 7 we had Bring Your Father to School Day, and one of my
classmates' father worked at Canonical and gave us all Ubuntu CDs. I installed
Ubuntu when I got home, which was pretty painless and easy, and it kind of just
worked. From then on, and for the last 14 years and counting, I've only used
Linux. I eventually transitioned from Ubuntu, to Arch, and then finally to
Gentoo where I've settled in the last little while.

So, to close up this introduction, in my 20 years on Earth I have barely used
Windows or macOS, and mostly used a variety of different Linux desktop
environments (Gnome 2 & 3, Mate, KDE 4 & 5, XFCE, LXDE, LXQT, i3, Sway) having
become quite comfortable with the Linux ecosystem over the years. About six
months ago I started a new job and was given a MacBook Pro, and so this story
began.

## The Good

I've always wanted to give macOS[^1] a try, it always looked beautiful and
polished to me. On top of that, there were certain applications that I had been
interested in trying for a while, and that only ran on macOS. Moreover, the
MacBook Pro's always seemed to me like the golden standard in hardware, they
were gorgeous and seemed to always work smoothly when I saw others use them.
Over the years I had grown to like the MacBooks so much I almost fantasized
about owning one.

Six months in, I can say that some of my admiration was well deserved. Overall I
find that the graphical experience on macOS is beautifully consistent, the
entire OS just feels coherent and nice to use. I just really like the look &
feel of macOS, especially Mojave with its comfortable Dark Mode.

Another thing that I really liked was having the Numbers/Pages/Keynote software
suit, it really does feel much better than LibreOffice. I wouldn't normally care
much for Pages or Keynote, but I found that my usual \\(\LaTeX\\) (and beamer)
workflow didn't work so well on macOS. Numbers is what I liked the most, not
having to use LibreOffice Calc was great. Overall the suite felt polished and
easy to use, except for the eventual annoying warnings about missing fonts on
documents.

One thing that I must give macOS absolute praise for is having consistent
keyboard shortcuts across applications. On Linux this is kind of a free-for-all
and becomes quite annoying with time, whereas Apple gets this almost entirely
right enforcing consistency across the OS.

Overall macOS feels well integrated, and mostly works well. Night mode (aka.
Redshift or f.lux) works well and starts to shift colors on my screen at
precisely the right times. Setting up Wi-Fi networks and Bluetooth devices is
super easy and also works well, I've gotten great range with my Sony Wh-1000XM2
when paired to the MacBook. Being able to take FaceTime calls on my computer
also felt pretty cool, especially when I was working and didn't want to leave my
desk to go pick up my phone in my room. Messages integration was OK.

As a last note on software, I want to touch on some things I didn't get to
interact much with. Firstly, APFS; it's good. As someone who deals with
filesystems quite a lot, I must say that APFS is an extremely solid CoW
offering. Native encryption and snapshotting just work, absolutely seamless
experience. I wish it had compression support, and that it was open source, but
nothing is perfect. Secondly Xcode; I barely used Xcode during my time with the
MacBook, mostly because IDEs just aren't my thing, but the few times I tried it
to test things out it seemed fine.

Now to the hardware: it's fantastic. The MacBook Pro[^2] feels absolutely
amazing when you hold it, and looks beautiful. I can confidently say this laptop
has the best look & feel of anything I've ever owned or used, period. The
keyboard, despite all the heavy criticism on the internet, was rather good in my
opinion, even though the ultra-short travel distance does make typing over long
periods of time a little exhausting. The backlight on the keyboard is nice, and
the brightness levels are appropriate, and I find that they often hit the
sweet-spot I needed for a variety of situations. The touchpad is truly
phenomenal, and miles ahead of anything anyone else has to offer; I never
thought a touchpad could feel this good. The screen is OK.

Another note on hardware that I really feel deserves its own section is audio,
because it's phenomenal. It is absolutely amazing to me that a laptop this thin
and this small could have audio this good. It's clear and gets loud with no
distortions, really just unbelievably good for a device in this form factor.

To close up this section, I hope I've made it clear that the MacBook isn't a
'steaming pile of garbage' as many on the Internet claim, there's a lot to love
here.

## The Bad

Unfortunately, the MacBook Pro, and macOS, are far from perfect, and there are a
few reasons I ultimately chose to leave it for a different machine.

Beginning with software once again, when macOS gets something right, it does it
to near perfection, but when it errs, oh does it err. For starters I've found
Finder to be an absolute steaming pile of crap. Everything just looks absolutely
chaotic on it, and the design feels so outdated. I tried setting up a network
share (with FTP) and it just did not work at all, and so many other small things
just felt clunky and bothersome that I've grown to dread it. On that note, what
is this absolute madness of having Finder always open and making it impossible
to close? I probably need Finder about 1% of the time, why should I have it
constantly open and cluttering my application switcher?[^3]

This brings me to application management on macOS, which is by far the thing I
hate the most about it, it's trash; full stop. macOS is the clunkiest, slowest,
most painful desktop experience I've used since the early days of Ubuntu's
Unity. Switching workspaces (aka. Desktops or Spaces) makes this slow and insane
animation that just makes me want to stab my computer. Eventually I figured out
how to make it far quicker, by enabling some accessibility setting, but it still
drives me insane. If that wasn't enough macOS will, through some obscure logic
that I am convinced came from Satan himself, reorder your workspaces so that
it's impossible to keep track of what is where when you are switching them
through keyboard shortcuts, or if mostly they just have a terminal open since
the text is far too tiny to be read from the 'switcher bar.' Eventually I
discovered how to disable this as well, but it still feels absolutely insane to
me that someone thought this improved usability somehow.

If the workspaces management wasn't bad enough, get ready for application
switching because it's another dumpster fire. Firstly, if you have two windows
open for a same application good old Alt+Tab (or Cmd+Tab in macOS) won't
work, that only switches between distinct applications; amazing. I eventually
mapped Cmd+Esc to switching between application windows. Moreover they apply the
same satanic algorithm from workspaces to dynamically reorder the applications
in the switcher, which makes it even worse. It's been months and I still haven't
been able to use workspaces or the damn switcher in a way that feels ergonomic
and productive. This has been the motivating force for me to move most of my
workflow into the terminal using tmux to manage panes, because if I need to
change workspaces or use the application switcher I will get angry.

In a similar area I find it absolutely counter-intuitive that when I close the
last window in some app it will remain "open" and clutter the already unusable
app switcher. So instead of Cmd+w doing the job, you have to Cmd+q and then apps
show you pop-ups when you are closing more than one window, how 2005. This
becomes supremely annoying when you open certain "one-shot" applications like
Preview for a quick glance at a PDF.

Now a brief intermission brought to you by macOS apps that absolutely don't cut
it. Preview is lacklustre and I find it unusable for anything that isn't a
simple viewing of a PDF. Spotlight is flaky, slow to detect newly installed
apps, and I find its search accuracy to be sub par. Siri's only use is to make
me angry when I click it as opposed to the hamburger menu that's right next to
it. The Dashboard hasn't been updated since Father Abraham walked this earth,
and it's also useless. Reminders is this weird mix between Calendar and Notes
that I don't get.

Back to our scheduled program, let's talk about window management in macOS; it's
also trash. I don't know what happened at Apple that caused them to have a
deathly fear of "snapping" windows, but this much needed feature is all but
missing from it, even Windows gets this right! I had to buy a $0.99 app called
Magnet (which is great) to get a minimally usable environment out of macOS.
Coming from a tiling window manager (Sway) I tried to use [chunkwm][chunkwm] for
a while but found it way too buggy and weird.

This leads us to package management on macOS, there are basically two kosher
routes: App Store, and HomeBrew. The App Store is OK, although it certainly
feels to me like the Ubuntu Software Center of old, and I find that it's kind of
a clunky UX. HomeBrew is macOS' third-party CLI package manager and I rate it a
solid 'meh.' I find that building the packages yourself, as opposed to
downloading them from the binary cache, is a bit of a second class citizen, as
well as the (in)ability to customize packages. It's written in Ruby, it's a bit
slow, but still faster than [Portage][portage]. Mostly, I just find having to
use two distinct package management solutions pretty awkward, and I found myself
multiple times wondering whether I installed something through Brew or through
some DMG.

Now that we've talked about packages, we should talk about POSIX tools on macOS.
Here's the deal, most scripts you find online have GNU POSIX tools in mind, and
macOS' tools have the same names (of course) but usually subtly different
options that break everything. The solution to this is to install coreutils
from HomeBrew, which installs `g`-prefixed versions of the usual tools, such as
`gmake`, and then patching your scripts.

Returning to my earlier point about keyboard shortcuts, one area of this that
Apple gets wrong are global shortcuts, they kind of suck. It's super easy to
have conflicting shortcuts with applications' defaults, and then there is no way
to set a preference for the OS defined ones. Also Cmd+Enter, which I used to
have bound to opening the terminal on Linux, is not a valid shortcut apparently,
which forced me to move to Cmd+\\. It's just annoying how shortcut conflicts &
interactions are handled.

Some other issues I had with macOS were weird video issues, my screen would
randomly glitch and flicker, albeit not too often, and I found Bootcamp, which I
needed a couple times, to be not nearly as good as I had hoped for. OpenVPN
support on macOS isn't great, although Tunnelblick is okay. You can't compile
kernel modules for macOS, so things like Wireguard run in userspace. Also,
DisplayLink drivers were broken on High Sierra and never really fixed, so my
dual monitor setup stopped working almost immediately after I got the computer.

Going on to hardware, for starters the components are a bit stunted if you want
a MacBook Pro without the ridiculous touch bar. The CPU only has four threads
(hyperthreaded), only has 16GB of RAM (the maximum available), but it does have
a remarkably fast SSD. Overall it just doesn't feel "pro," and there's really
nowhere to run if you want to have a real Esc key. If that wasn't enough, I've
discovered that my machine will randomly contact NASA to request a liftoff and
starts running its fans at absolutely full blast, even though nothing is going
on and I only have a single iTerm window open. At least the fans aren't loud,
even if they do have a particularly annoying high-pitched sound. Curiously, if
you click the Siri button it immediately throttles the fans down and cancels its
impending trip to Mars.

Together with the Yuri Gagarin fans on my machine I also got really horrible
battery life. I've always liked MacBooks for their notoriously good batteries,
alas I have found this to absolutely not be the case. My remaining battery
estimates are mostly untrustworthy, and I probably can't get more than four
hours on this MacBook. Also, I'm on board for the "USB-C all things!" bandwagon,
despite the serious flaws of USB-C, but having only two ports is absolutely not
enough, and please give me better ports, these are so flimsy connectors will
pretty much just slide off them.

The crown jewel of hardware goofs on this laptop is most definitely the keyboard
however. After only six months, my `2` is almost completely stuck and my
spacebar will randomly lock and spam whatever text-box I have on focus. Even
though I was prepared for this with all the criticism this keyboard has received
and used an external keyboard almost the entire time I wasn't immune to the
failures.

As a final note on hardware, there is no or poor Linux support, depending on
which exact model you have. I tried to get Linux on it a few times when I got
sick of macOS and failed to do so in any usable form.

Overall some parts of macOS and the MacBook Pro are really bad, and in
particular I found that they had a deep impact on my productivity. In the six
months I've owned this machine, it has made me absolutely hate using my computer
for the most part. It got to a point where I would have macOS solely running
VMWare with Linux on it.

## The Ugly

Honestly there is not much to say here, one thing the MacBook Pro and macOS get
right is looks. Most of the OS looks great, except for some minor exceptions I
mentioned in the previous section, and the hardware really is gorgeous and feels
high-quality.

Where other laptops struggle to make thin, beautiful laptops that look modern,
Apple succeeds. While Linux desktop environments lag behind on their look &
feel, macOS succeeds.

## Conclusion

If not for the major flaws I outlined, the MacBook Pro would be an absolutely
killer offering in the "pro-market." I find it to be a tremendous shame that
Apple gets so many difficult things right, and then absolutely fails to provide
usable solutions to the simpler problems. I think mainly macOS is a huge
deterrent of the MacBook's potential, I find that it's just unusable for me, and
the inability to just switch to Linux makes the entire laptop kind of useless.

As for me, I am about to end my adventure through the world of Macs and going to
a [Thinkpad P1][thinkpad][^4]. I think it's a much better suited machine for my
needs both software (Linux) and hardware (compiling) wise. I am a bit sad to let
go off the MacBook however, I did like a lot about it, but it just wasn't for
me.

## Afterthoughts

I wonder whether my criticisms are perhaps ill-founded or unwarranted, since I
made the unusual switch of a highly customized and comfortable Linux environment
to macOS. I'm not entirely sure whether that is the case, but either way I think
some of the points I raised are independent of that and valid regardless of it.

Also, I hope I do not come off as an Apple or MacBook hater, I really wanted to
love this machine and to love macOS, I just couldn't.

[^1]: Note that I am specifically referring to my experience using macOS High
    Sierra and Mojave.
[^2]: Late 2017 model, 13", no touch bar.
[^3]: I know I can disable that and make it closable, and I have, but still it's
    hidden in some obscure setting that must be enabled through some terminal
    command, what is this, GNOME?
[^4]: With a Xeon E-2176M, Quadro P2000, 4K screen, 32GB of DDR4, and a 1TB NVME
    SSD.

[chunkwm]: https://koekeishiya.github.io/chunkwm
[portage]: https://wiki.gentoo.org/wiki/Portage
[thinkpad]: https://www.lenovo.com/us/en/laptops/thinkpad/thinkpad-p/ThinkPad-P1/p/22WS2WPP101
