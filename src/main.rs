use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ray_tracing::render::{ppm, Color, Image};
use setup::{IMAGE_HEIGHT, IMAGE_WIDTH, REPETITION};

mod setup;

fn create_image() -> thread::Result<Vec<Color>> {
    // ProgressBar
    let mp = MultiProgress::new();

    const DRAW_RATE: u64 = 15;

    let sty = ProgressStyle::default_bar().template(
        "{spinner} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {percent}% ~{eta}",
    );

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

    let data = thread::spawn(move || {
        let res = setup::run(pb_run.clone(), pb_curr.clone());
        for pb in [pb_curr, pb_run] {
            pb.finish();
        }

        ab.store(false, Ordering::Release);

        res
    });

    mp.join().map_err(|err| Box::new(err) as _)?;
    ticker.join()?;

    data.join()
}

fn main() {
    // Image
    let path = "main";

    println!("Running");

    let data = create_image().expect("unable to get the data, due to some error");

    println!("Writing data");
    let img = Image::new(&data, IMAGE_HEIGHT, IMAGE_WIDTH);

    ppm::save(img, path).expect("Something went terribly wrong here");
    println!("Done");
}
