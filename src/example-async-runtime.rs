use chrono::prelude::*;
use tokio::runtime;
use tokio::time::Duration;

use futures::stream::StreamExt;

fn main() {
    let max_tasks = 100_usize;
    let max_threads_runtime = 4_usize;
    let max_threads_scheduling_for_stream = 4_usize; // mirar efecto con o sin tokio sleep
    let runtime = runtime::Builder::new_multi_thread()
        .worker_threads(max_threads_runtime)
        .enable_time()
        .on_thread_start(|| {
            println!("thread worker started");
        })
        .build()
        .unwrap();
    runtime.block_on(async {
        let _ = futures::stream::iter(0..max_tasks)
            .map(|n| {
                tokio::spawn(async move {
                    let utc: DateTime<Utc> = Utc::now();
                    println!(
                        "time: {} value: {} thread-id: {:?}",
                        utc,
                        n,
                        std::thread::current().id()
                    );
                    // aca no se bloquearia ninguna task, ejecutan todas, y al hacer await, pollea a otra!
                    //  es la hermosa visualizacion de la concurrencia con programacion asincronica!
                    // tokio::time::sleep(Duration::from_secs(2)).await; 

                    // computacion INTENSA detro de una task! se bloquearia el thread del worker !!! 
                    std::thread::sleep(Duration::from_secs(2)); 
                })
            })
            .buffer_unordered(max_threads_scheduling_for_stream)
            .collect::<Vec<_>>()
            .await;
    });
}
