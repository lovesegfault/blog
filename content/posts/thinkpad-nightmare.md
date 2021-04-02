+++
title = "Thinkpad Nightmare"
date = 2019-01-24T23:38:15Z
draft = false
tags = [ "lenovo", "thinkpad", "bios", "nightmare" ]
categories = [ "essay" ]
comments = true
+++

A few weeks ago I finally got my new work computer. I had been using an
underpowered MacBook Pro that I had grown to hate for months, and so I asked for
a Linux laptop, and another systems engineer followed along. When they asked me
which laptop I would like, I recommended a Thinkpad. I had heard nothing but
good things about them and their Linux support, and so I felt confident they
would prove to be a good choice. I ended up going for the Xeon variant of the
Lenovo P1, a laptop costing roughly $3000.

The day I got the machine I was beaming with excitement, "Finally I will be able
to use a tiling window manager again!" I booted it up, allowed Windows to
perform its updates and rebooted. After that I allowed Lenovo Vantage, their
update/management software to search for updates, and sure enough it found some
updates and drivers. I let it run, and in the end it prompted me to update my
BIOS. I had read online about multiple issues regarding the 1.15 version that my
machine had shipped with, such as bricking when certain options were enabled, so
I allowed it to run. Some shell command executed, and then the machine promptly
rebooted. The BIOS update utility showed up, and started doing its thing, until
the progress bar stopped. One hour passed, and then another, and then five, and
the bar did not move. I searched around online and it did not seem like anyone
was experiencing this, so I called Lenovo support.

Here I really want to stop for a moment to emphasize just how remarkably
terrible Lenovo support is. Firstly, going through the menus is an advanced game
of 4-D chess against the robotic voices that guide you through the maze of
options. In fact, in one of my attempts I even managed to reach an infinite loop
where it was impossible to go back, and all the options were invalid. When,
after completing the dungeon and passing through IBM and then back to Lenovo,
you finally speak to a human, you are welcomed by the Lenovo technical support
center in Atlanta, Georgia[^1]. Now, I am not a native speaker of English, but
my (spoken) English is almost undetectable, most people think I'm from Ohio and
I've had a hard time convincing some otherwise, and yet every single person I
spoke to at the support center was almost unintelligible. I really could not
understand what they were saying, which made it all the more frustrating. To cut
this short, I eventually managed to communicate, at which point they told me to
unplug the computer and allow it to power off eventually and see what happened.
My computer was bricked.

With an expensive piece of metal sitting on my desk, I called Lenovo support
again, and was informed that it would take up to a month for it to be repaired.
I chose to go for a replacement. The replacement eventually came in, and I did
the same update process, except this time it worked fine, yippie!

The machine mostly worked fine with Linux, except for the fact that I had some
weird issue where it would shut down randomly, it was rare enough it didn't
bother me all that much. Sure enough, I went to check and there was another BIOS
update fixing the shutdown issue. I took a Xanax[^2], prepared for the worst,
and performed the update. Surprisingly, it worked perfectly fine. Or so I
though, somehow in the update Power on AC seemed to have been activated, which I
found rather annoying, so I went ahead and disabled that in the BIOS. After that
my machine never booted again. So here I am, again, with an expensive piece of
metal.

I didn't dare call Lenovo support this time, I value my sanity too much, instead
I just contacted our vendor CDW[^3] and asked for a replacement, which I should get
some time soon. In the mean time I am stuck with the MacBook that I hate and
with unfinished, valuable, work inside of the Thinkpad that failed me again.

I wish I could recommend Lenovo and this Thinkpad, but if they can't get
something as basic and as essential as their BIOS right they might as well give
up on making computers. I just hope I get my replacement soon, before Chinese
New Years so it doesn't take even longer so that I can go back to work soon.

[^1]: I only know this because an annoying voice, almost mocking you after the
    journey it took to reach the support center, announces it proudly.

[^2]: Not really.

[^3]: The unsung hero here is our CDW representative, Shane, who is the most
    timely, professional, and helpful person I have ever dealt with in this area.
    I could not recommend CDW more.
