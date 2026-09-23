//! Création device D3D11 + factory DXGI.

use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1};

use crate::error::CaptureError;

pub(crate) struct D3dDevice {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
}

pub(crate) fn create_factory() -> Result<IDXGIFactory1, CaptureError> {
    unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }.map_err(map_win)
}

pub(crate) fn create_device_for_adapter(
    adapter: &IDXGIAdapter1,
) -> Result<D3dDevice, CaptureError> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let mut level = D3D_FEATURE_LEVEL_11_0;

    unsafe {
        D3D11CreateDevice(
            adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut level),
            Some(&mut context),
        )
    }
    .map_err(map_win)?;

    let device = device.ok_or_else(|| CaptureError::Native("D3D11 device null".into()))?;
    let context = context.ok_or_else(|| CaptureError::Native("D3D11 context null".into()))?;
    Ok(D3dDevice { device, context })
}

pub(crate) fn map_win(err: windows::core::Error) -> CaptureError {
    CaptureError::Native(err.to_string())
}
