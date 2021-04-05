use std::io::Result;
use std::mem;

#[cfg(target_os = "linux")]
pub fn get_cores() -> Result<Vec<usize>> {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("/proc/cpuinfo")?;
    let reader = BufReader::new(file);
    let mut map = BTreeMap::new();
    let mut procid = 0;
    let mut physid = 0;
    let mut coreid = 0;

    for line in reader.lines() {
        let line = line?;
        if line.len() == 0 {
            map.entry((physid, coreid))
                .or_insert(Vec::new())
                .push(procid);
            continue;
        }
        let mut iter = line.split(':');
        let (key, value) = match (iter.next(), iter.next()) {
            (Some(key), Some(value)) => (key.trim(), value.trim()),
            _ => panic!("cannot parse /proc/cpuinfo"),
        };
        if key == "processor" {
            procid = value.parse().expect("cannot parse processor id");
            physid = 10000 + physid;
            coreid = 10000 + coreid;
        }
        if key == "physical id" {
            physid = value.parse().expect("cannot parse physical id");
        }
        if key == "core id" {
            coreid = value.parse().expect("cannot parse core id");
        }
    }

    Ok(map.into_iter().map(|(_, v)| *v.first().unwrap()).collect())
}

#[cfg(unix)]
pub fn set_affinity(core: usize) {
    unsafe {
        let mut set: libc::cpu_set_t = mem::zeroed();
        libc::CPU_SET(core, &mut set);
        libc::sched_setaffinity(0, mem::size_of_val(&set), &set);
    }
}

#[cfg(windows)]
pub fn get_cores() -> Result<Vec<usize>> {
    use std::ptr;
    use winapi::um::sysinfoapi::GetLogicalProcessorInformation;
    use winapi::um::winnt::{
        RelationProcessorCore, SYSTEM_LOGICAL_PROCESSOR_INFORMATION as ProcInfo,
    };
    unsafe {
        let len = {
            let mut len = 0;
            GetLogicalProcessorInformation(ptr::null_mut(), &mut len);
            len as usize / mem::size_of::<ProcInfo>()
        };

        let mut vec = Vec::with_capacity(len);
        vec.set_len(len);

        let mut len2 = (len * mem::size_of::<ProcInfo>()) as u32;
        let ret = GetLogicalProcessorInformation(vec.as_mut_ptr(), &mut len2);
        assert!(ret != 0);

        Ok(vec
            .into_iter()
            .filter(|v| v.Relationship == RelationProcessorCore)
            .map(|v| v.ProcessorMask.trailing_zeros() as usize)
            .collect())
    }
}

#[cfg(windows)]
pub fn set_affinity(core: usize) {
    use winapi::um::processthreadsapi::GetCurrentThread;
    use winapi::um::winbase::SetThreadAffinityMask;
    unsafe {
        let mask = 1 << core;
        SetThreadAffinityMask(GetCurrentThread(), mask);
    }
}
