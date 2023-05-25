#![allow(dead_code, unused_variables)]

use std::{io::{self, Read}, path::{Path, PathBuf}, fs};

use blend::MyBlend;
use color_name::Color;
use dialoguer::{Select, Input, Confirm};
use fs_extra::dir::CopyOptions;
use image::{imageops::{FilterType, self}, RgbaImage, Rgba};
use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};
use ini::{Ini, WriteOption, EscapePolicy, LineSeparator};
use native_dialog::FileDialog;
use regex::Regex;
use std::process;

mod blend;

macro_rules! exit {
  ($s:expr) => {
    let mut stdin = io::stdin();

    println!($s);
    println!("Press any key to exit");

    let _ = stdin.read(&mut [0u8]).unwrap();

    process::exit(0);
  };
}

struct FontsConfig {
  prefix: String,
  overlay_above_number: bool,
}

type SkinColour = (u8, u8, u8);

const EMPTY_IMAGE: &'static [u8] = include_bytes!("empty.png");

fn colorize_image(image: &mut MyBlend<RgbaImage>, w: u32, h: u32, color: SkinColour) {
  let color = Rgba([color.0, color.1, color.2, 100]);

  draw_filled_rect_mut(image, Rect::at(0, 0).of_size(w, h), color);
}

fn generate_images(
  path: &PathBuf,
  fonts_config: &FontsConfig,
  colour: Option<SkinColour>,
  is_circle_hd: bool,
  is_circle_overlay_hd: bool,
  is_numbers_hd: bool
) -> Vec<RgbaImage> {
  let mut images = Vec::new();

  let mut overlay = if is_circle_overlay_hd {
    let mut overlay = image::open(path.join("hitcircleoverlay@2x.png")).unwrap();
    if !(is_circle_hd && is_numbers_hd) {
      overlay = overlay.resize(overlay.width() / 2, overlay.height() / 2, FilterType::Nearest);
    }
    overlay
  } else {
    image::open(path.join("hitcircleoverlay.png")).unwrap()
  };

  overlay = overlay.resize((overlay.width() as f32 * 1.25) as u32, (overlay.height() as f32 * 1.25) as u32, FilterType::CatmullRom);

  let mut hitcircle = if is_circle_hd {
    let mut hitcircle = image::open(path.join("hitcircle@2x.png")).unwrap();
    if !(is_circle_overlay_hd && is_numbers_hd) {
      hitcircle = hitcircle.resize(hitcircle.width() / 2, hitcircle.height() / 2, FilterType::Nearest);
    }
    hitcircle
  } else {
    image::open(path.join("hitcircle.png")).unwrap()
  };

  hitcircle = hitcircle.resize((hitcircle.width() as f32 * 1.25) as u32, (hitcircle.height() as f32 * 1.25) as u32, FilterType::CatmullRom);

  let (hcw, hch) = (hitcircle.width(), hitcircle.height());
  let mut hitcircle = MyBlend(RgbaImage::from_vec(hitcircle.width(), hitcircle.height(), hitcircle.to_rgba8().to_vec()).unwrap());

  if let Some(colour) = colour {
    colorize_image(&mut hitcircle, hcw, hch, colour);
  }

  let hitcircle = hitcircle.0;

  for i in 0..=9 {
    let number = if is_numbers_hd {
      let mut number = image::open(path.join(format!("{}-{}@2x.png", fonts_config.prefix, i))).unwrap();
      if !(is_circle_hd && is_circle_overlay_hd) {
        number = number.resize(number.width() / 2, number.height() / 2, FilterType::Nearest);
      }
      number
    } else {
      image::open(path.join(format!("{}-{}.png", fonts_config.prefix, i))).unwrap()
    };

    let (w, h) = (
      hitcircle.width().max(overlay.width()),
      hitcircle.height().max(overlay.height())
    );

    let mut image = RgbaImage::new(w, h);

    imageops::overlay(
      &mut image,
      &hitcircle,
      (w / 2 - hitcircle.width() / 2) as i64,
      (h / 2 - hitcircle.height() / 2) as i64
    );

    if fonts_config.overlay_above_number {
      imageops::overlay(
        &mut image,
        &number,
        (w / 2 - number.width() / 2) as i64,
        (h / 2 - number.height() / 2) as i64
      );
    }

    imageops::overlay(
      &mut image,
      &overlay,
      (w / 2 - overlay.width() / 2) as i64,
      (h / 2 - overlay.height() / 2) as i64
    );

    if !fonts_config.overlay_above_number {
      imageops::overlay(
        &mut image,
        &number,
        (w / 2 - number.width() / 2) as i64,
        (h / 2 - number.height() / 2) as i64
      );
    }

    images.push(image);
  }

  images
}

fn generate_skin(
  path: &PathBuf,
  name: String,
  is_circle_hd: bool,
  is_circle_overlay_hd: bool,
  is_numbers_hd: bool,
  ini: &mut Ini,
  fonts_config: FontsConfig,
  colour: Option<SkinColour>
) {
  println!("Generating numbers");

  let numbers = generate_images(path, &fonts_config, colour, is_circle_hd, is_circle_overlay_hd, is_numbers_hd);

  let new_path = path.clone().parent().unwrap().join(name);

  println!("Cloning skin folder");

  fs::create_dir(new_path.clone()).unwrap();
  match fs_extra::dir::copy(path.clone(), new_path.clone(), &CopyOptions::new().content_only(true)) {
    Err(err) => {
      exit!("Unable to clone the skin");
    },
    _ => {}
  };

  println!("Replacing files");

  if Path::is_file(new_path.join("hitcircle@2x.png").as_path()) {
    fs::remove_file(new_path.join("hitcircle@2x.png")).unwrap();
  }
  if Path::is_file(new_path.join("hitcircle.png").as_path()) {
    fs::remove_file(new_path.join("hitcircle.png")).unwrap();
  }

  if Path::is_file(new_path.join("hitcircleoverlay@2x.png").as_path()) {
    fs::remove_file(new_path.join("hitcircleoverlay@2x.png")).unwrap();
  }
  if Path::is_file(new_path.join("hitcircleoverlay.png").as_path()) {
    fs::remove_file(new_path.join("hitcircleoverlay.png")).unwrap();
  }

  if Path::is_file(new_path.join("sliderstartcircle@2x.png").as_path()) {
    fs::remove_file(new_path.join("sliderstartcircle@2x.png")).unwrap();
  }
  if Path::is_file(new_path.join("sliderstartcircle.png").as_path()) {
    fs::remove_file(new_path.join("sliderstartcircle.png")).unwrap();
  }

  if Path::is_file(new_path.join("sliderstartcircleoverlay@2x.png").as_path()) {
    fs::remove_file(new_path.join("sliderstartcircleoverlay@2x.png")).unwrap();
  }
  if Path::is_file(new_path.join("sliderstartcircleoverlay.png").as_path()) {
    fs::remove_file(new_path.join("sliderstartcircleoverlay.png")).unwrap();
  }

  fs::write(new_path.join("hitcircle.png"), EMPTY_IMAGE).unwrap();
  fs::write(new_path.join("hitcircleoverlay.png"), EMPTY_IMAGE).unwrap();

  let hd = is_circle_hd && is_circle_overlay_hd && is_numbers_hd;

  for i in 0..=9 {
    if Path::is_file(new_path.join(format!("{}-{}@2x.png", fonts_config.prefix, i)).as_path()) {
      fs::remove_file(new_path.join(format!("{}-{}@2x.png", fonts_config.prefix, i))).unwrap();
    }
    if Path::is_file(new_path.join(format!("{}-{}.png", fonts_config.prefix, i)).as_path()) {
      fs::remove_file(new_path.join(format!("{}-{}.png", fonts_config.prefix, i))).unwrap();
    }

    if hd {
      numbers[i].save(new_path.join(format!("{}-{}@2x.png", fonts_config.prefix, i))).unwrap();
    } else {
      numbers[i].save(new_path.join(format!("{}-{}.png", fonts_config.prefix, i))).unwrap();
    }
  }

  let colour = match colour {
    Some(colour) => colour,
    None => (255, 255, 255)
  };

  ini.with_section(Some("Fonts"))
    .set("HitCircleOverlap", format!("{}", if hd { numbers[0].width() / 2 } else { numbers[0].width() }));
  
  ini.with_section(Some("Colours"))
    .delete(&"Combo1").delete(&"Combo2").delete(&"Combo3").delete(&"Combo4")
    .delete(&"Combo5").delete(&"Combo6").delete(&"Combo7").delete(&"Combo8")
    .set("Combo1", format!("{}, {}, {}", colour.0, colour.1, colour.2));

  ini.write_to_file_opt(new_path.join("skin.ini"), WriteOption {
    escape_policy: EscapePolicy::Basics,
    line_separator: LineSeparator::SystemDefault,
    kv_separator: ": ",
  }).unwrap();

  exit!("Instafade has been generated");
}

fn main() {
  let path = FileDialog::new()
    .show_open_single_dir()
    .unwrap();

  let path = match path {
    None => {
      exit!("No skin selected. Terminating.");
    },
    Some(path) => path,
  };

  let mut ini = match Ini::load_from_file(path.join("skin.ini")) {
    Ok(ini) => ini,
    Err(_) => {
      exit!("Unable to parse skin.ini");
    }
  };

  let fonts_config = match ini.section(Some("Fonts")) {
    None => FontsConfig {
      prefix: "default".to_owned(),
      overlay_above_number: true,
    },
    Some(fonts) => {
      let prefix = match fonts.get("HitCirclePrefix") {
        Some(prefix) => prefix.to_owned(),
        None => "default".to_owned(),
      };

      let overlay_above_number = match fonts.get("HitCircleOverlayAboveNumber") {
        Some(above) => above == "1",
        None => false
      };

      FontsConfig {
        prefix,
        overlay_above_number
      }
    }
  };

  let colours: Vec<SkinColour> = match ini.section(Some("Colours")) {
    None => vec![
      (255, 192, 0),
      (0, 202, 0),
      (18, 124, 255),
      (242, 24, 57),
    ],
    Some(colours) => {
      let mut c = Vec::new();

      let re = Regex::new(r"(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})").unwrap();

      for i in 1..=8 {
        if let Some(combo) = colours.get(format!("Combo{}", i)) {
          let color = re.captures(combo).unwrap();

          c.push((
            str::parse::<u8>(color.get(1).unwrap().as_str()).unwrap(),
            str::parse::<u8>(color.get(2).unwrap().as_str()).unwrap(),
            str::parse::<u8>(color.get(3).unwrap().as_str()).unwrap(),
          ));
        }
      }

      if c.len() > 0 {
        c
      } else {
        vec![
          (255, 192, 0),
          (0, 202, 0),
          (18, 124, 255),
          (242, 24, 57),
        ]
      }
    }
  };

  let name: String = Input::new()
    .with_prompt("Skin name")
    .with_initial_text(format!("{} - instafade", path.file_name().unwrap().to_str().unwrap()))
    .interact_text()
    .unwrap();

  if Path::is_dir(path.parent().unwrap().join(&name).as_path()) {
    if Confirm::new().with_prompt("A skin with this name already exists. Replace it?").interact().unwrap() {
      fs_extra::dir::remove(path.parent().unwrap().join(&name)).unwrap();
    } else {
      exit!("Canceled");
    }
  }

  let colour = {
    let items: Vec<String> = colours.iter().map(|c| format!("{} ({}, {}, {})", Color::similar([ c.0, c.1, c.2 ]), c.0, c.1, c.2)).collect();

    let selection = Select::new()
      .with_prompt("Choose the colour")
      .item("Default")
      .items(&items)
      .default(0)
      .interact()
      .unwrap();

    if selection == 0 {
      None
    } else {
      Some(colours[selection - 1])
    }
  };

  let is_circle_hd = Path::is_file(path.join("hitcircle@2x.png").as_path());
  let is_circle_overlay_hd = Path::is_file(path.join("hitcircleoverlay@2x.png").as_path());
  let is_numbers_hd = Path::is_file(path.join(format!("{}-0@2x.png", fonts_config.prefix)).as_path());

  generate_skin(
    &path,
    name,
    is_circle_hd,
    is_circle_overlay_hd,
    is_numbers_hd,
    &mut ini,
    fonts_config,
    colour
  );
}
