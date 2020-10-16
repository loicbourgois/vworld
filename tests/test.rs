extern crate arrayvec;
use arrayvec::ArrayVec;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::ptr;

fn main() {
    let thread_count = 10;


    let (tx, rx) = mpsc::channel();

    let channels: [(std::sync::mpsc::Sender<u32>, std::sync::mpsc::Receiver<u32>); 2] = [mpsc::channel(), mpsc::channel()];

    // ;
    //let (tx2, rx2) = mpsc::channel();
    let channels_plus = ArrayVec::from(channels);

    //let senders: [std::sync::mpsc::Sender<u32>; 2] = [channels[0].0, channels[1].0];
    //let senders: [std::sync::mpsc::Sender<u32>; 2] = [channels[0].0, channels[1].0];

    let tx_b = tx.clone();

    //let rec = channels_plus[0].1;


    let (tx3, rx3) = mpsc::channel();
    let (tx4, rx4) = mpsc::channel();
    let receivers = [rx3, rx4];

    let mut channels_vec: Vec<(std::sync::mpsc::Sender<u32>, std::sync::mpsc::Receiver<u32>)> = Vec::new();
    for i in 0..thread_count {
        channels_vec.push(mpsc::channel());
    }

    let aa = unsafe { ptr::read(&channels_vec[0].1) };

    thread::spawn(move || {
        aa.recv().unwrap();
        aa.recv().unwrap();

        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];
        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });
    thread::spawn(move || {

        //receivers[1].recv().unwrap();
        //receivers[1].recv().unwrap();

        let vals = vec![
            String::from("zf"),
            String::from("zrgzv"),
            String::from("zefzef"),
            String::from("zefzef"),
        ];
        for val in vals {
            tx_b.send(val).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    channels_plus[0].0.send(2).unwrap();
    channels_vec[0].0.send(2).unwrap();
    tx3.send(2).unwrap();

    for received in rx {
        println!("Got: {}", received);
    }
}
