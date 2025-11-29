use anyhow::Result;
use libmpv2::Mpv;
use libmpv2_sys::{
    mpv_render_context, mpv_render_context_create, mpv_render_context_free,
    mpv_render_context_render, mpv_render_context_update, mpv_render_param,
    mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
    mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_FORMAT,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_POINTER,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_SIZE,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_STRIDE,
    mpv_render_update_flag_MPV_RENDER_UPDATE_FRAME,
};
use std::ffi::{CString, c_void};
use std::ptr;

pub struct VideoRenderer {
    ctx: *mut mpv_render_context,
}

unsafe impl Send for VideoRenderer {}
unsafe impl Sync for VideoRenderer {}

impl VideoRenderer {
    pub fn new(mpv: &Mpv) -> Result<Self> {
        unsafe {
            let mut ctx: *mut mpv_render_context = ptr::null_mut();

            let api_type = CString::new("sw").unwrap();

            let mut params = vec![
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                    data: api_type.as_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                    data: ptr::null_mut(),
                },
            ];

            let mpv_ctx = mpv.ctx.as_ptr();
            let err = mpv_render_context_create(&mut ctx, mpv_ctx, params.as_mut_ptr());
            if err < 0 {
                return Err(anyhow::anyhow!("Failed to create render context: {}", err));
            }

            Ok(Self { ctx })
        }
    }

    pub fn update(&mut self) -> bool {
        unsafe {
            let flags = mpv_render_context_update(self.ctx);
            (flags & (mpv_render_update_flag_MPV_RENDER_UPDATE_FRAME as u64)) != 0
        }
    }

    pub fn render_sw(
        &mut self,
        width: i32,
        height: i32,
        stride: i32,
        format: &str,
        buffer: &mut [u8],
    ) -> Result<()> {
        unsafe {
            let format_c = CString::new(format).unwrap();
            let mut size = [width, height];
            let stride_val = stride; // Need a variable to take address of

            let mut params = vec![
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_SIZE,
                    data: size.as_mut_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_FORMAT,
                    data: format_c.as_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_STRIDE,
                    data: &stride_val as *const _ as *mut c_void,
                },
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_POINTER,
                    data: buffer.as_mut_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                    data: ptr::null_mut(),
                },
            ];

            let err = mpv_render_context_render(self.ctx, params.as_mut_ptr());
            if err < 0 {
                return Err(anyhow::anyhow!("Failed to render: {}", err));
            }
        }
        Ok(())
    }
}

impl Drop for VideoRenderer {
    fn drop(&mut self) {
        unsafe {
            mpv_render_context_free(self.ctx);
        }
    }
}
