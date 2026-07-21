use buddy_system_allocator::LockedHeap;

//follow the rules to provide ABIs for allocator
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    framework::init_heap_allocator(&HEAP_ALLOCATOR);
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout)
}
