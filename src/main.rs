use std::{
    panic,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};
use ray_tracing::{Config, render::{self, Color, Image}};
use scenes::{WorldSettings, scenes::Worlds};


pub const REPETITION: usize = 1;
const WORLD: Worlds = Worlds::FinalScene;

pub fn run(
    WorldSettings { conf, world, cam }: &WorldSettings,
    pb_run: ProgressBar,
    pb_int: ProgressBar,
) -> anyhow::Result<Vec<Color>> {
    pb_run.set_position(0);

    // run
    // SAFETY: the unwrap is safe here as we know
    // that there allways will be a result.
    let mut res = (0..REPETITION)
        .map(|_| ray_tracing::run(world, &conf, pb_int.clone(), &cam))
        .progress_with(pb_run)
        .reduce(|mut acc, v| {
            for (a, b) in acc.iter_mut().zip(v.iter()) {
                *a += *b;
            }
            acc
        })
        .unwrap();

    let len = REPETITION as f64;
    for val in res.iter_mut() {
        *val /= len;
    }

    Ok(res)
}

pub fn create_image() -> anyhow::Result<(Config, Vec<Color>)> {
    // setup render
    let settings =  scenes::setup(WORLD)?;
    let conf = settings.conf.clone();

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
    let pb_curr = setup(conf.image_height());

    let ab = Arc::new(AtomicBool::new(true));

    let ticker = {
        let pb_run1 = pb_run.clone();
        let pb_curr1 = pb_curr.clone();

        let ab1 = ab.clone();

        thread::spawn(move || {
            let s = Duration::from_millis(1000 / DRAW_RATE);

            while ab1.load(Ordering::Acquire) {
                pb_run1.tick();
                pb_curr1.tick();
                thread::sleep(s);
            }
        })
    };

    let mp_handler = thread::spawn(move || mp.join());

    let data = thread::spawn(move || {
        let res = run(&settings, pb_run.clone(), pb_curr.clone());

        for pb in [pb_curr, pb_run] {
            pb.finish();
        }

        ab.store(false, Ordering::Release);

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
        Err(err) => panic::resume_unwind(err),
    }

    let tmp = res?;

    Ok((conf, tmp))
}

fn main() {
    // Image
    let path = "main";

    println!("Running");

    let (conf, data) = create_image().expect("unable to get the data, due to some error");

    println!("Writing data");
    let img = Image::new(&data, conf.image_height(), conf.image_width());

    render::save(img, path, render::FileFormat::PNG).expect("Something went terribly wrong here");
    println!("Done");
}
