use std::{panic, sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }, thread, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ray_tracing::render::{self, Color, Image};
use setup::{IMAGE_HEIGHT, IMAGE_WIDTH, REPETITION};

mod scenes;
mod setup;

fn create_image() -> anyhow::Result<Vec<Color>> {
    // ProgressBar
    let mp = MultiProgress::new();

    const DRAW_RATE: u64 = 15;

    // Calculate the size of the current terminal
    let (_, size) = console::Term::stdout().size();
    let size = if size < 70 { size / 3 } else { size / 2 };

    let format = format!("{{spinner}} [{{elapsed_precise}}] {{bar:{}.cyan/blue}} {{pos:>7}}/{{len:7}} {{percent}}% ~{{eta}}", size);

    let sty = ProgressStyle::default_bar().template(&format);

    let setup = |size| {
        let pb = mp.add(ProgressBar::new(size as u64));
        pb.set_style(sty.clone());
        pb.set_draw_rate(DRAW_RATE);
        pb
    };

    let pb_run = setup(REPETITION);
    let pb_curr = setup(IMAGE_HEIGHT);

    let pb_run1 = pb_run.clone();
    let pb_curr1 = pb_curr.clone();

    let ab = Arc::new(AtomicBool::new(true));
    let ab1 = ab.clone();

    let ticker = thread::spawn(move || {
        let s = Duration::from_millis(1000 / DRAW_RATE);

        while ab1.load(Ordering::Acquire) {
            pb_run1.tick();
            pb_curr1.tick();
            thread::sleep(s);
        }
    });

    let mp_handler = thread::spawn(move || mp.join());

    let data = thread::spawn(move || {
        let res = setup::run(pb_run.clone(), pb_curr.clone());

        for pb in [pb_curr, pb_run] {
            pb.finish();
        }

        ab.store(false, Ordering::Release);

        // as we are returning a Box<dyn error::Error> we can not move to to the
        // parent thread :(
        res
    });


    // special case of a panic happening
    let res = match data.join() {
        Ok(res) => res,
        Err(err) => panic::resume_unwind(err),
    };

    if let Err(err) = ticker.join() {
        panic::resume_unwind(err);
    }

    match mp_handler.join() {
        Ok(res) => res?,
        Err(err) => panic::resume_unwind(err)
    }

    res
}

fn main() {
    // Image
    let path = "main";

    println!("Running");

    let data = create_image().expect("unable to get the data, due to some error");

    println!("Writing data");
    let img = Image::new(&data, IMAGE_HEIGHT, IMAGE_WIDTH);

    render::save(img, path, render::FileFormat::PNG).expect("Something went terribly wrong here");
    println!("Done");
}
