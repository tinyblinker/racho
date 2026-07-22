use crate::config::*;
use crate::trap::TrapContext;

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, trap_cx: TrapContext) -> usize {
        let trap_cx_ptr = (self.get_sp() - size_of::<TrapContext>()) as *mut TrapContext;
        *framework::phys_to_ref(trap_cx_ptr as usize) = trap_cx;
        trap_cx_ptr as usize
    }
}
impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

/// get total number of the application
pub fn get_num_app() -> usize {
    framework::get_num_app()
}

/// Get the base address of the i-th application
pub fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

/// Load the user app at
/// [APP_BASE_ADDRESS + n * APP_SIZE_LIMIT, APP_BASE_ADDRESS + (n + 1) * APP_SIZE_LIMIT]).
pub fn load_apps() {
    let num_app = get_num_app();
    for i in 0..num_app {
        let base_i = get_base_i(i);
        framework::zero_memory(base_i, APP_SIZE_LIMIT);
        let app_data = framework::get_app_data(i);
        let dst = framework::phys_to_byte_slice(base_i, app_data.len());
        dst.copy_from_slice(app_data);
        framework::fence_i();
    }
}

/// Get application data (raw ELF bytes)
pub fn get_app_data(app_id: usize) -> &'static [u8] {
    framework::get_app_data(app_id)
}
