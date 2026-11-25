# Other people's work that the site is made with: Graphviz, KaTeX, and the open fonts.
{
  lib,
  dejavu_fonts,
  fetchurl,
  graphviz,
  katex,
  makeWrapper,
  python3,
  runCommand,
}:
let
  fetch =
    path: hash:
    fetchurl {
      url = "https://raw.githubusercontent.com/google/fonts/6ebe737e44fd64ee736bbc6feed6136a6dbfd222/ofl/${path}";
      inherit hash;
    };

  # Nearly every page gets by on the first file. The second holds the rest of the font.
  common = "U+0-17F,U+2C6-2DD,U+2000-206F,U+20AC,U+2122,U+2190-2199,U+2212,U+FFFD";
  rare = "U+180-2C5,U+2DE-1FFF,U+2070-20AB,U+20AD-2121,U+2123-218F,U+219A-2211,U+2213-2BFF,U+FB00-FB4F";

  face = family: weight: style: path: hash: ''
    name=${lib.removeSuffix ".ttf" (baseNameOf path)}
    cp ${fetch path hash} fonts/$name.ttf
    for part in common=${common} rare=${rare}; do
      pyftsubset fonts/$name.ttf --unicodes=''${part#*=} --output-file=fonts/$name-''${part%=*}.woff2 --flavor=woff2 \
        --no-hinting --name-IDs='*' --layout-features=calt,case,ccmp,kern,liga,lnum,locl,mark,mkmk,onum,pnum,tnum
      echo "@font-face { font-family: '${family}'; font-weight: ${weight}; font-style: ${style}; font-display: swap;" \
        "src: url(fonts/$name-''${part%=*}.woff2); unicode-range: ''${part#*=}; }" >> fonts.css
    done
  '';
in
runCommand "vendor"
  {
    nativeBuildInputs = [
      makeWrapper
      (python3.withPackages (python: [
        python.brotli
        python.fonttools
      ]))
    ];
  }
  ''
    mkdir -p $out/fonts $out/katex/fonts $out/ruler
    cd $out

    # Graphviz measures labels with whatever font it finds, so it is left a monospaced one.
    cp ${dejavu_fonts}/share/fonts/truetype/DejaVuSansMono.ttf ruler
    echo "<fontconfig><dir>$out/ruler</dir></fontconfig>" > ruler/fonts.conf
    makeWrapper ${graphviz}/bin/dot dot --set FONTCONFIG_FILE $out/ruler/fonts.conf
    katex=${katex}/lib/node_modules/katex/dist
    cp $katex/katex.min.js katex
    cp $katex/fonts/*.woff2 katex/fonts
    sed -E 's/,url\([^)]*\.(woff|ttf)\) format\("[a-z]+"\)//g' $katex/katex.min.css > katex/katex.min.css

    ${face "Spectral" "300" "normal" "spectral/Spectral-Light.ttf"
      "sha256-oqUwMD0yZHO2mreGO4eekgPsdH5R1fp8exng6XXgB0A="
    }
    ${face "Spectral" "300" "italic" "spectral/Spectral-LightItalic.ttf"
      "sha256-TLQkrrHWuCLpzmITeJr+q9QDxnZfxzOeGEwnrk1qawE="
    }
    ${face "Spectral" "400" "normal" "spectral/Spectral-Regular.ttf"
      "sha256-yJAh3CByDI0Nz0CwsvbgDBNmX6gEFxf1gTlvUbjHj10="
    }
    ${face "Spectral" "400" "italic" "spectral/Spectral-Italic.ttf"
      "sha256-fslyRCWdtACMSxIkx5FOU3HHl8AESvnYXC12G6Dl94c="
    }
    ${face "Spectral" "600" "normal" "spectral/Spectral-SemiBold.ttf"
      "sha256-X4aRWnRIMuz25KF6sEvqCRufqZLvUWT/Za40wdov6Us="
    }
    ${face "Spectral" "600" "italic" "spectral/Spectral-SemiBoldItalic.ttf"
      "sha256-Rq06n5O5jcLb3u4mXnbrhGxb6RKqjwn0LjaScGxvVjs="
    }
    ${face "Bona Nova" "400" "normal" "bonanova/BonaNova-Regular.ttf"
      "sha256-1y93Fba2YJbgzClx5JVPi+M73O0fFD+cVzmgPNtgvt4="
    }
    ${face "Bona Nova" "400" "italic" "bonanova/BonaNova-Italic.ttf"
      "sha256-DWHBYKUhsHLRrxnpaDdU8/V2G7zjX3P98JHs30lbxsI="
    }
  ''
