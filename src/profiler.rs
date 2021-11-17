use pprof::{protos::Message, ProfilerGuard};
use std::fs::File;
use std::io::Write;

pub struct Profiler {
    inner: Option<ProfilerGuard<'static>>,
    frequency: i32,
}

impl Profiler {
    pub fn new(frequency: i32) -> Self {
        Self {
            inner: Some(pprof::ProfilerGuard::new(frequency).unwrap()),
            frequency,
        }
    }

    pub fn reset(&mut self) {
        let profiler = std::mem::take(&mut self.inner).unwrap();
        std::mem::drop(profiler);
        self.inner = Some(pprof::ProfilerGuard::new(self.frequency).unwrap())
    }

    pub fn report_then_reset(&mut self, path: &str) {
        let profiler = std::mem::take(&mut self.inner).unwrap();

        let profile = profiler.report().build().unwrap().pprof().unwrap();
        let mut content = Vec::new();
        profile.encode(&mut content).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&content).unwrap();

        std::mem::drop(profiler);
        self.inner = Some(pprof::ProfilerGuard::new(self.frequency).unwrap())
    }
}
