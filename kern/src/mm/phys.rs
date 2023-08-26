use alloc::{collections::BTreeMap, format, sync::Arc, vec::Vec};
use fdt::node::FdtNode;
use spin::Mutex;

use crate::{
    arch, conf, driver,
    error::{InternalError, Result},
    heap, info,
    mm::virt::VirtAddr,
};

use core::{
    fmt::Display,
    ops::{Add, Bound, Sub},
    slice,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(usize);

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = usize;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl PhysAddr {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn align_page_down(self) -> Self {
        Self(self.0 & !(conf::PAGE_SIZE - 1))
    }

    pub fn align_page_up(self) -> Self {
        Self((self.0 + conf::PAGE_SIZE - 1) & !(conf::PAGE_SIZE - 1))
    }

    pub fn as_usize(self) -> usize {
        self.0
    }

    pub fn as_array_base<T>(self) -> &'static mut [T] {
        let addr = self.align_page_down().as_usize();
        unsafe {
            slice::from_raw_parts_mut(addr as *mut T, conf::PAGE_SIZE / core::mem::size_of::<T>())
        }
    }
}

pub(super) struct PhysFrameAllocator {
    regions: BTreeMap<PhysAddr, usize>,
    init_state: Vec<(PhysAddr, usize)>,
}

impl PhysFrameAllocator {
    const fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            init_state: Vec::new(),
        }
    }

    fn append_region(&mut self, regions: (PhysAddr, usize)) {
        self.merge_or_insert(regions);
    }

    fn remove_region(&mut self, regions: (PhysAddr, usize)) {
        let (addr, size) = regions;
        let Some((l_addr, l_size)) = self
            .regions
            .upper_bound_mut(Bound::Included(&addr))
            .remove_current()
        else {
            panic!("no such physical memory region: {} - {}", addr, addr + size);
        };

        let size = size + (addr - addr.align_page_down());
        let addr = addr.align_page_down();
        let size = size + ((addr + size).align_page_up() - (addr + size));

        if addr - l_addr >= conf::PAGE_SIZE {
            self.regions
                .try_insert(l_addr, addr.align_page_down() - l_addr)
                .unwrap();
        }
        if (l_addr + l_size) - (addr + size) >= conf::PAGE_SIZE {
            self.regions
                .try_insert(addr + size, l_addr + l_size - addr - size)
                .unwrap();
        }
    }

    fn set_init_regions(&mut self) {
        for region in self.regions.iter() {
            self.init_state.push((*region.0, *region.1));
        }
    }

    fn get_init_regions(&self) -> slice::Iter<'_, (PhysAddr, usize)> {
        self.init_state.iter()
    }

    fn alloc(&mut self) -> Result<PhysAddr> {
        let (addr, size) = self
            .regions
            .first_entry()
            .ok_or(InternalError::NotEnoughMem)?
            .remove_entry();
        if addr != addr.align_page_down() {
            panic!("unaligned physical address: {}", addr);
        }
        if size < conf::PAGE_SIZE {
            Err(InternalError::NotEnoughMem)
        } else {
            if size > conf::PAGE_SIZE {
                self.regions
                    .try_insert(addr + conf::PAGE_SIZE, size - conf::PAGE_SIZE)
                    .unwrap();
            }
            Ok(addr)
        }
    }

    fn dealloc(&mut self, addr: PhysAddr) -> Result<()> {
        if addr != addr.align_page_down() {
            panic!("unaligned physical address: {}", addr);
        }
        self.merge_or_insert((addr, conf::PAGE_SIZE));
        Ok(())
    }

    fn merge_or_insert(&mut self, pending: (PhysAddr, usize)) {
        fn merge_left(
            regions: &mut BTreeMap<PhysAddr, usize>,
            pending: (PhysAddr, usize),
        ) -> (PhysAddr, usize) {
            let (addr, size) = pending;
            if let Some((l_addr, l_size)) = regions
                .upper_bound_mut(Bound::Included(&addr))
                .remove_current()
            {
                if addr < l_addr + l_size {
                    panic!(
                        "overlapped physical memory regions: {} - {}",
                        addr,
                        l_addr + l_size
                    );
                }
                if l_addr + l_size == addr {
                    regions.try_insert(l_addr, l_size + size).unwrap();
                    (l_addr, l_size + size)
                } else {
                    regions.try_insert(l_addr, l_size).unwrap();
                    pending
                }
            } else {
                pending
            }
        }

        fn merge_right(
            regions: &mut BTreeMap<PhysAddr, usize>,
            pending: (PhysAddr, usize),
        ) -> (PhysAddr, usize) {
            let (addr, size) = pending;
            if let Some((r_addr, r_size)) = regions
                .lower_bound_mut(Bound::Included(&(addr + size)))
                .remove_current()
            {
                if addr + size > r_addr {
                    panic!(
                        "overlapped physical memory regions: {} - {}",
                        addr,
                        r_addr + r_size
                    );
                }
                if addr + size == r_addr {
                    regions.try_insert(addr, size + r_size).unwrap();
                    (addr, size + r_size)
                } else {
                    regions.try_insert(r_addr, r_size).unwrap();
                    pending
                }
            } else {
                pending
            }
        }

        let (addr, size) = pending;

        let pending = merge_left(&mut self.regions, pending);
        let pending = merge_right(&mut self.regions, pending);
        if pending == (addr, size) {
            self.regions.try_insert(addr, size).unwrap();
        }
    }
}

static PHYS_FRAME_ALLOCATOR: Mutex<PhysFrameAllocator> = Mutex::new(PhysFrameAllocator::new());

fn probe(node: &FdtNode) -> Result<()> {
    extern "C" {
        fn _end();
    }
    let mut allocator = PHYS_FRAME_ALLOCATOR.lock();
    for mem_region in node.reg().ok_or(InternalError::DevProbeError)? {
        let addr = PhysAddr::new(mem_region.starting_address as usize);
        let size = mem_region.size.ok_or(InternalError::DevProbeError)?;
        info!("probed physical memory region: {} - {}", addr, addr + size);
        allocator.append_region((addr, size));
    }
    for (addr, size) in arch::layout::get_prot_addrs() {
        info!("probed protected memory region: {} - {}", addr, addr + size);
        allocator.remove_region((arch::mm::virt_to_phys(addr), size));
    }
    allocator.set_init_regions();
    info!(
        "usable physical memory regions: {}",
        allocator
            .get_init_regions()
            .map(|&(addr, size)| { format!("{} - {}", addr, addr + size) })
            .collect::<Vec<_>>()
            .join(",")
    );
    let heap_start = VirtAddr::new(_end as usize).align_page_up();
    let heap_size = (heap_start.align_page_up()
        + (allocator
            .regions
            .iter()
            .map(|(_, &size)| size)
            .sum::<usize>()
            / 128))
        .align_page_up()
        - heap_start;
    info!(
        "enlarge heap with memory region: {} - {}",
        heap_start,
        heap_start + heap_size,
    );
    let heap_region = (heap_start, heap_size);
    heap::enlarge(heap_region);
    allocator.remove_region((arch::mm::virt_to_phys(heap_region.0), heap_region.1));
    Ok(())
}

pub fn init() {
    driver::register! {
        devtyp("memory") => probe
    };
}

pub(super) fn get_init_regions() -> Vec<(PhysAddr, usize)> {
    let mut vec = Vec::new();
    PHYS_FRAME_ALLOCATOR
        .lock()
        .get_init_regions()
        .collect_into(&mut vec);
    vec
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame {
    addr: PhysAddr,
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        PHYS_FRAME_ALLOCATOR.lock().dealloc(self.addr).unwrap();
    }
}

impl PhysFrame {
    pub fn alloc() -> Result<Arc<Self>> {
        let addr = PHYS_FRAME_ALLOCATOR.lock().alloc()?;
        addr.as_array_base::<u8>().fill(0);
        Ok(Arc::new(Self { addr }))
    }

    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}
