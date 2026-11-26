# The fonts, whole for Typst and in parts for browsers.
{
  fetchFromGitHub,
  python3,
  runCommand,
  spectral,
}:
let
  bona-nova = fetchFromGitHub {
    owner = "google";
    repo = "fonts";
    rev = "6ebe737e44fd64ee736bbc6feed6136a6dbfd222";
    rootDir = "ofl/bonanova";
    hash = "sha256-LbvzFjk1M6MRnVVfo32zPy04dXe9db77TT1NAKApLJw=";
  };
in
runCommand "fonts"
  {
    nativeBuildInputs = [
      (python3.withPackages (python: [
        python.brotli
        python.fonttools
      ]))
    ];
  }
  ''
    mkdir -p $out/mono
    cd $out
    fonttools ttLib.woff2 decompress ${../assets/fonts/BerkeleyMono.woff2} -o mono/BerkeleyMono.ttf

    # Nearly every page gets by on the first part. The second holds the rest of the font.
    common=U+0-17F,U+2C6-2DD,U+2000-206F,U+20AC,U+2122,U+2190-2199,U+2212,U+FFFD
    rare=U+180-2C5,U+2DE-1FFF,U+2070-20AB,U+20AD-2121,U+2123-218F,U+219A-2211,U+2213-2BFF,U+FB00-FB4F

    face() {
      cp $4 .
      for part in common rare; do
        file=$(basename $4 .ttf)-$part.woff2
        pyftsubset $4 --unicodes=''${!part} --output-file=$file --flavor=woff2 --no-hinting --name-IDs='*' \
          --layout-features=calt,case,ccmp,kern,liga,lnum,locl,mark,mkmk,onum,pnum,tnum
        echo "@font-face { font-family: '$1'; font-weight: $2; font-style: $3; font-display: swap;" \
          "src: url($file); unicode-range: ''${!part}; }" >> fonts.css
      done
    }

    face Spectral 300 normal ${spectral}/share/fonts/truetype/Spectral-Light.ttf
    face Spectral 300 italic ${spectral}/share/fonts/truetype/Spectral-LightItalic.ttf
    face Spectral 400 normal ${spectral}/share/fonts/truetype/Spectral-Regular.ttf
    face Spectral 400 italic ${spectral}/share/fonts/truetype/Spectral-Italic.ttf
    face Spectral 600 normal ${spectral}/share/fonts/truetype/Spectral-SemiBold.ttf
    face Spectral 600 italic ${spectral}/share/fonts/truetype/Spectral-SemiBoldItalic.ttf
    face "Bona Nova" 400 normal ${bona-nova}/BonaNova-Regular.ttf
    face "Bona Nova" 400 italic ${bona-nova}/BonaNova-Italic.ttf
  ''
