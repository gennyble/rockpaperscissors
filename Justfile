resvg_args := "--width 24 --height 24"

make-assets:
	resvg {{resvg_args}} assets/rock.svg assets/rock.png
	resvg {{resvg_args}} assets/paper.svg assets/paper.png
	resvg {{resvg_args}} assets/scissors.svg assets/scissors.png