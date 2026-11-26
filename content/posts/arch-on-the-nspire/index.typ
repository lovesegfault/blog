#import "/blog.typ": *

#show: post.with(
  title: [Running Arch on the TI Nspire CX Calculator],
  date: "2016-10-28T00:00:00Z",
  categories: "tutorial",
  tags: ("linux", "ti", "arm", "calculator"),
)

#let alarm = link.with("https://archlinuxarm.org")
#let debian-nspire = link.with("/posts/debian-on-the-nspire/")
#let ndless-sdk = link.with(
  "https://github.com/ndless-nspire/Ndless/wiki/Ndless-SDK:-C-and-assembly-development-introduction",
)
#let bash-src = link.with("./files/setpath.sh")
#let nspire-kernel = link.with("https://github.com/lovesegfault/nspire-kernel")
#let linux = link.with(
  "https://www.kernel.org/pub/linux/kernel/v4.x/linux-4.3.tar.xz",
)
#let linuxpkg = link.with("./files/linux-armv5-4.3.0-1-arm.pkg.tar.xz")
#let headerspkg = link.with(
  "./files/linux-armv5-headers-4.3.0-1-arm.pkg.tar.xz",
)
#let distcc = link.with("https://wiki.archlinux.org/index.php/Distcc")
#let pkgbuild = link.with("https://github.com/archlinuxarm/PKGBUILDs/")
#let ndless = link.with("http://ndless.me/")
#let tiplanet = link.with("https://tiplanet.org/forum/ndl3ss.php")
#let bootloader = link.with("./files/linuxloader2.tns")
#let startscript = link.with("./files/start_usb.ll2.tns")

This is a follow up to my previous post on how to run Debian on the Nspire,
this time we will be going deeper into the matter and compiling the Linux Kernel
ourselves, and doing such in a way that it's compatible with
#alarm[Arch Linux ARM]

= Hardware requirements

The requirements for this project are the same as for the
#debian-nspire[last one], here's what you'll need:

- A Texas TI Nspire CX or CX CAS
- USB Hub
- Mini-B OTG USB Cable
- USB Drive

= Setting everything up

Since we will be building the Kernel ourselves this time we will need to do some
special work on our setup, in particular to our build environment.

== Cross Compiling

Firstly, we must get a cross-compiler working, in this case an `arm-none-eabi`
type compiler, there's a caveat though, in order for the Kernel to work with
the Nspire we must build it on an outdated toolchain. There is a good tutorial
over on #ndless-sdk[Ndless SDK] on how to do that, but we will be going over
the steps here.

+ Clone the SDK
  - `git clone --recursive https://github.com/ndless-nspire/Ndless.git`

+ Run the `ndless-sdk/toolchain/build_toolchain.sh` script.
  - This will download the toolchain as well as build it for you. It will
    take a while.

  #picture(
    "images/compiling.jpg",
    alt: "Compiler output scrolling by, next to htop with every core busy",
  )[Compiling is tough on the CPU]

+ The original guide recommends adding the binaries' path to your `PATH`, but
  since I don't think that amending your `.bashrc` for one-time projects like
  these is worth it, and having to manually edit your path can be boring,
  I have created this #bash-src[`bash` script]
  that does that. Place it inside the `Ndless` folder, and then source it
  (make sure your current work path is the `Ndless` folder too, i.e. `cd`
  into it).

+ Your cross-compiler should be working now. Test it by running
  `arm-none-eabi-gcc` and see if you get any output.

  #picture(
    "images/xcompiler.jpg",
    alt: "arm-none-eabi-gcc failing with: no input files",
  )[You should get output like this]

If you managed to produce output in step 4. that means your Cross Compiler is
working well. Remember to source that file whenever you want to use the
binaries you compiled, or add them to your `PATH` on `.bashrc` if you think
you might be using them often.

=== Getting the kernel

Every kernel compilation starts with a configuration file, this one is no
different. Since setting the configuration so that it works on the Nspire is a
hassle, I have set up #nspire-kernel[a repository] that will contain
configurations for different versions of the Linux kernel. At the time of
writing, solely `4.3.0` is there, but work on other versions is ongoing. Feel
free to contact me if you'd like a config for a version of the kernel not yet
supported. For the rest of this guide we will be using Kernel `4.3.0`, although
nothing should change for future versions.

+ Download the Linux kernel and untar it
  - #linux[`linux-4.3.tar.xz`]

+ Clone the configurations repository
  - `git clone https://github.com/lovesegfault/nspire-kernel`

+ Copy the configuration file to the kernel folder, name it .config
  - `cp nspire-kernel/4.3.0/config linux-4.3/.config`

#picture(
  "images/config.jpg",
  alt: "A terminal showing the first lines of the kernel's .config",
)[You should be able to get output like this]

With this your kernel is ready for compilation, don't worry, we will get to it
in a moment.

= Compiling the kernel

Now that we have set up all the pieces that we will need for the
compilation, we can start it. Make sure you have your .config file in the right
place. Also confirm that you have the toolchain we compiled in your `PATH`.

+ `ARCH=arm make -jX`
  - `ARCH=arm` specifies the architecture we're building to
  - `-jX` parallelizes compilation. Replace `X` with your thread count.

  #picture(
    "images/linux-compiling.jpg",
    alt: "The kernel being built, next to htop",
  )[Compiling the kernel, get yourself a cup of coffee]

+ Check that you have the following files
  - `arch/arm/boot/zImage`
  - `arch/arm/boot/dts/cx.dtb`

The compilation step is now complete. We have built our kernel image (`zImage`)
and out device tree file (`nspire-cx.dtb`) which are the files we needed.

= Preparing the system drive

In my last post we set op our rootfs using debian tools, this time it will
be a bit different. Up until now this guide was distro-independent, apart
from the fact that the config file is heavily influence by arch linux's one,
but we must shift this perspective now to focus on the subject of this post:
Arch Linux. For the rest of this project we will be using #alarm[ALARM], the
Arch Linux port for the ARM architecture.

== Getting the files

+ Download the latest version of ALARM for ARMv5
  - http://os.archlinuxarm.org/os/ArchLinuxARM-armv5-latest.tar.gz

+ Make sure your USB drive uses an MBR partition table

+ Format the drive. Add some SWAP space, it will be useful for doing more
  memory-hungry operations.
  - I'm using 512MiB Swap space on my drive, reason for which is the fact
    that my drive is small at 8GB.

+ Untar the file you downloaded into the drive
  - `sudo bsdtar xzf ArchLinuxARM-armv5-latest.tar.gz -C ~/mnt/`
    Replace `~/mnt` with wherever you mounted your drive.

    #picture(
      "images/rootfs.jpg",
      alt: "A terminal listing the top of a Linux root filesystem",
    )[Your rootfs should look like this]

== Installing modules

+ `cd` into the directory where we compiled linux

+ Make sure the `arm-none-eabi` toolchain is in your `PATH`

+ Install modules to the USB drive
  - `sudo ARCH=arm make modules_install INSTALL_MOD_PATH=~/mnt/`

#picture(
  "images/modules.jpg",
  alt: "A terminal listing installed modules, ending in DEPMOD 4.3.0-nspire",
)[Modules installed]

== Changing root with QEMU

+ Make sure you have `qemu-user-static` and `binfmt-support` installed.

+ Copy QEMU binaries to the drive: `sudo cp /usr/bin/qemu-arm-static  ~/mnt/usr/bin`

+ Register `qemu-arm-static` as an ARM interpreter in the kernel (must be
  root)

  #code(title: "register-qemu.sh", wrap: true)[
    ```bash
    echo ':arm:M::\x7fELF\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x28\x00:\xff\xff\xff\xff\xff\xff\xff\x00\xff\xff\xff\xff\xff\xff\xff\xff\xfe\xff\xff\xff:/usr/bin/qemu-arm-static:' > /proc/sys/fs/binfmt_misc/register
    ```
  ]

+ Change root into the new rootfs. If you are running arch and you have the
  `arch-install-scripts` package installed you can use `arch-chroot`

+ Synchronize package databases `pacman -Syy`

+ At this point you might want to install some recommended packages, such as
  `sudo`. If you have a USB WiFi Adapter connected to your hub you should
  also consider installing `dialog` and `wpa_supplicant`

== Downgrading the kernel

Since we are using an outdated kernel version, `4.3.0`, relative to the one
on the ALARM image we extracted we must downgrade the kernel. Firstly you will
need a kernel package. You can download the ones I compiled
(#linuxpkg[`linux-armv5-4.3.0-1-arm.pkg.tar.xz`] and
#headerspkg[`linux-armv5-headers-4.3.0-1-arm.pkg.tar.xz`]) and install
from the `chroot` with `pacman -U [FILES]`, or compile it yourself. You can do
that with either #distcc[`distcc`] (recommended) or with the following steps:

+ Clone the ALARM #pkgbuild[PKGBUILD repository]

+ Roll it back to commit `c82145d0d491c2e216ff49ec60d3e83c13e73230`
  - `git reset --hard c82145d0d491c2e216ff49ec60d3e83c13e73230`

+ Copy the `core/linux-armv5` folder somewhere else

+ Replace the `config` file in the folder with our own

+ Get an MD5Sum of our config file

+ Replace the MD5Sum on the `PKGBUILD` file with out new one, it's the last
  one in the list.

+ `makepkg -cs --install`

#picture(
  "images/downgrade.jpg",
  alt: "pacman asking to downgrade linux-armv5 from 4.7.6-1 to 4.3.0-1",
)[Downgrade in progress]

Whichever way you choose, you can check if it worked by:

- Running `ls /lib/modules` and seeing if you see the correct module folder
  there.
- Running `pacman -Qs` and checking if it lists the kernel as version
  `4.3.0-1`

#picture(
  "images/check-downgrade.jpg",
  alt: "A terminal listing the 4.3.0 module folders, and pacman reporting linux-armv5 4.3.0-1",
)[Downgrade successful]

Lastly, add the packages `linux-armv5` and `linux-armv5-headers` to your ignore
list over on `/etc/pacman.conf`. To do so, uncomment the `IgnorePkg` line, and
add them, separated by spaces, to it. This will guarantee that these changes
won't be later overwritten by some system upgrade.

== Setting up the calculator

To run the bootloader for the Linux kernel you will need to
#ndless[jailbreak] your calculator. There are a lot of good online tutorials on
how to do this, I, for one, recommend the guide from #tiplanet[TIPlanet].
With that out of the way we can get to setting up the files on the calculator's
flash memory.

+ Download the #bootloader[bootloader]

+ Download the #startscript[starting script]

+ `cd` into the folder containing Linux, where we performed the compilation
  steps.

+ Copy `arch/arm/boot/zImage` and `arch/arm/boot/dts/cx.dtb` somewhere

+ Concatenate the `.tns` extension to both of them

Since the TI Connectivity software does not work on Linux at all (not even
under Wine), you will require either a Windows or MacOS machine to get
files to and from your calculator, Virtual Machines work.

+ Edit the `ndless.cfg.tns` file and append `ext.ll2 linuxloader2` to it.
  - This associates our starting script to the bootloader program

+ Create a folder named `linux`

+ Place the files you downloaded, `zImage.tns`, and `nspire-cx.dtb.tns` into
  the `linux` folder

#picture(
  "images/files.jpg",
  alt: "The TI-Nspire file browser, with four files in the linux folder",
)[Files on the calculator]

== Finishing up

Once the files have been placed on the calculator, the process is done. You may now
start the OS by executing the bootloader. If you have any questions on how to
do this you can reference the #debian-nspire[initial article]. Below is a
demo video of the calculator playing Doom on Linux.

#video("https://www.youtube.com/watch?v=O3eSspki0Ws")[
  The calculator playing Doom on Linux
]
