use std::thread;
use std::time::SystemTime;
use std::time::Duration;

use std::sync::Arc;
use std::sync::Mutex;

/// CronoCounter es un cronometro/contador.
/// La idea es que simule un cronometro, pero la actualizacion se tiene que hacer
/// manualmente. Work es un booleano medio agarrado de los pelos,
/// que se utiliza en Clock

#[derive(Clone)]
struct CronoCounter {

    now: SystemTime,
    work: bool,
}

impl CronoCounter {

    pub fn new() -> CronoCounter{

        CronoCounter {
            now: SystemTime::now(),
            work: true,
        }

    }

    pub fn update(&mut self) -> bool{

        self.now = self.now.checked_add(Duration::new(1, 0)).unwrap();
        self.work

    }

    pub fn get(&self) -> SystemTime {
        self.now.clone()
    }

    pub fn stop(&mut self) {
        self.work = false;
    }

}

/// Clock es el cronometro en si.
/// Posee un timer: un proceso paralelo que loopea indefinidamente
/// (bueno en realidad si esta definido cuando detenerse).
/// El loop duerme un segundo, y actualiza el cronocounter.
/// Si en algun momento le digo stop() al cronocounter, timer
/// va a detenerse y cerrara su proceso (como esta implementado
/// en Drop).

pub struct Clock {
    crono: Arc<Mutex<CronoCounter>>,
    timer: Option<thread::JoinHandle<()>>,
}

impl Clock {

    pub fn new() -> Clock {

        let cronocounter = Arc::new(Mutex::new(CronoCounter {
            now: SystemTime::now(),
            work: true,
        }));

        let cronocounter_clone = Arc::clone(&cronocounter);
        let timer = thread::spawn(move || {
            
            loop {
                if !cronocounter_clone.lock().unwrap().update() {
                    println!("Shutting down clock");
                    break;
                }
                thread::sleep(Duration::new(1, 0));
            }

        });

        Clock {
            crono: cronocounter,
            timer: Some(timer),
        }

    }

    fn update(&mut self){

        self.crono.lock().unwrap().update();

    }

    pub fn get_now(&self) -> SystemTime {

        self.crono.lock().unwrap().get()

    }

}

impl Drop for Clock {

    fn drop(&mut self) {
        self.crono.lock().unwrap().stop();
        if let Some(thread) = self.timer.take() {
            thread.join().unwrap();
        }
        println!("Clock was shutted down");
    }

}

#[cfg(test)]
mod test_clock {

    use super::*;

    #[test]
    fn test01_waiting_five_seconds(){

        let mut clock = Clock::new();
        let time1 = clock.get_now();
        thread::sleep(Duration::new(5, 0));
        let time2 = clock.get_now();
        let diff = time2.duration_since(time1).unwrap();
        //println!("IMPRIMIIIII LA CONCHA DE TU MADRE: {:?}", diff.as_secs_f32());
        assert!(4.99 < diff.as_secs_f32());
        assert!(diff.as_secs_f32() < 5.01);

    }

}

