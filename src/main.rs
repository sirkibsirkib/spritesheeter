use image::DynamicImage;
use image::GenericImageView;
use image::Rgba;

struct MyImage {
    img: DynamicImage,
    wh: [u32; 2],
}

fn main() {
    let matches = clap::App::new("My Super Program")
		.version("1.0")
		.author("Christopher Esterhuyse. <christopher.esterhuyse@gmail.com>")
		.about("Builds a spritesheet png out of a set of pngs")
		.arg(clap::Arg::with_name("t")
			.help("Source images are placed at the topleft of their sprites (default is centering).")
			.short("t")
			.long("topleft"))
		.arg(clap::Arg::with_name("OUTPUT")
			.help("Output path")
			.short("o")
			.long("output")
			.value_name("OUTPUT")
			.takes_value(true)
			.required(true))
		.arg(clap::Arg::with_name("INPUT")
			.help("Input png paths separated by spaces. Minimum 1.")
			.multiple(true)
			.min_values(1)
			.required(true))
		.get_matches();

    println!("current dir: {:?}", std::env::current_dir().unwrap());
    let imgs = matches
        .values_of("INPUT")
        .unwrap()
        .map(|path| {
            let mut img = image::open(&path).expect(&format!("No valid file at {:?}", &path));
            auto_crop(&mut img)
        })
        .collect::<Vec<_>>();
    let mut max_wh = [0, 0];
    for img in imgs.iter() {
        for i in 0..=1 {
            max_wh[i] = max_wh[i].max(img.wh[i]);
        }
    }

    let tot_wh = [max_wh[0] * (imgs.len() as u32), max_wh[1]];
    let offset_centering = |img: &MyImage| {
        let mut offset = [0; 2];
        for i in 0..=1 {
            offset[i] = (max_wh[i] - img.wh[i]) / 2;
        }
        offset
    };
    let offset_tl = |_img: &MyImage| [0, 0];
    let offset_func: &(dyn Fn(&MyImage) -> [u32; 2]) = match matches.occurrences_of("t") {
        0 => &offset_centering,
        _ => &offset_tl,
    };

    let offsets = imgs.iter().map(offset_func).collect::<Vec<_>>();
    println!("OFFSETS: {:?}", &offsets);
    println!("sprite dims: {:?}", max_wh);
    println!("sheet dims : {:?}", tot_wh);
    let new = image::ImageBuffer::from_fn(tot_wh[0], tot_wh[1], |x, y| {
        let idx = x / max_wh[0];
        let offset = offsets[idx as usize];
        let px = (x % max_wh[0]) as i32 - offset[0] as i32;
        let py = (y % max_wh[1]) as i32 - offset[1] as i32;
        let im = imgs.get(idx as usize).unwrap();
        if px >= 0 && px < im.wh[0] as i32 && py >= 0 && py < im.wh[1] as i32 {
            im.img.get_pixel(px as u32, py as u32)
        } else {
            Rgba { data: [0, 0, 0, 0] }
        }
    });
    let out_pth = matches.value_of("OUTPUT").unwrap();
    new.save(out_pth)
        .expect(&format!("Provided output {:?} path gave error", out_pth));
}

fn auto_crop(img: &mut DynamicImage) -> MyImage {
    if let Some(r_img) = img.as_rgba8() {
        let (w, h) = r_img.dimensions();
        let mut left = 0;
        let mut right = w;
        let mut top = 0;
        let mut bottom = h;
        'outer1: for x in 0..w {
            for y in 0..h {
                if r_img.get_pixel(x, y).data[3] > 0 {
                    left = x;
                    break 'outer1;
                }
            }
        }
        'outer2: for x in (0..w).rev() {
            for y in 0..h {
                if r_img.get_pixel(x, y).data[3] > 0 {
                    right = x;
                    break 'outer2;
                }
            }
        }
        'outer3: for y in 0..h {
            for x in 0..w {
                if r_img.get_pixel(x, y).data[3] > 0 {
                    top = y;
                    break 'outer3;
                }
            }
        }
        'outer4: for y in (0..h).rev() {
            for x in 0..w {
                if r_img.get_pixel(x, y).data[3] > 0 {
                    bottom = y;
                    break 'outer4;
                }
            }
        }
        let img2 = img.crop(left, top, right - left, bottom - top);
        MyImage {
            img: img2,
            // xy: [left, top],
            wh: [right - left, bottom - top],
        }
    } else {
        panic!("BAD FMT??");
    }
}
