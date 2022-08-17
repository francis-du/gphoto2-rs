//! Camera related stuff

use crate::{
  abilities::Abilities,
  file::CameraFilePath,
  helper::{camera_text_to_str, uninit},
  port::PortInfo,
  try_gp_internal,
  widget::{Widget, WidgetType},
  Result,
};
use std::{borrow::Cow, ffi, marker::PhantomData, os::raw::c_char};

/// Represents a camera
pub struct Camera<'a> {
  pub(crate) camera: *mut libgphoto2_sys::Camera,
  pub(crate) context: *mut libgphoto2_sys::GPContext,
  _phantom: PhantomData<&'a ffi::c_void>,
}

impl Drop for Camera<'_> {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_camera_unref(self.camera);
      libgphoto2_sys::gp_context_unref(self.context);
    }
  }
}

impl<'a> Camera<'a> {
  pub(crate) fn new(
    camera: *mut libgphoto2_sys::Camera,
    context: *mut libgphoto2_sys::GPContext,
  ) -> Self {
    Self { camera, context, _phantom: PhantomData }
  }

  /// Capture image
  ///
  /// ## Returns
  ///
  /// A [`CameraFilePath`] which can be downloaded to the host system
  // TODO: Usage example
  pub fn capture_image(&self) -> Result<CameraFilePath> {
    let mut file_path_ptr = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_capture(
      self.camera,
      libgphoto2_sys::CameraCaptureType::GP_CAPTURE_IMAGE,
      &mut file_path_ptr,
      self.context
    ))?;

    Ok(file_path_ptr.into())
  }

  /// Get the camera's [`Abilities`]
  pub fn abilities(&self) -> Result<Abilities> {
    let mut abilities = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_abilities(self.camera, &mut abilities))?;

    Ok(abilities.into())
  }

  /// Summary of the cameras model, settings, capabilities, etc.
  pub fn summary(&self) -> Result<Cow<str>> {
    let mut summary = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_summary(
      self.camera,
      &mut summary,
      self.context
    ))?;

    Ok(camera_text_to_str(summary))
  }

  /// Port used to connect to the camera
  pub fn port_info(&self) -> Result<PortInfo> {
    let mut port_info = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_port_info(self.camera, &mut port_info))?;

    Ok(PortInfo { inner: port_info })
  }

  /// Get about information about the camera#
  pub fn about(&self) -> Result<Cow<str>> {
    let mut about = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_about(self.camera, &mut about, self.context))?;

    Ok(camera_text_to_str(about))
  }

  /// Get the camera configuration
  pub fn config(&self) -> Result<Widget<'a>> {
    let mut root_widget = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_config(
      self.camera,
      &mut root_widget,
      self.context
    ))?;

    Ok(Widget::new(root_widget))
  }

  /// Get a single configuration by name
  pub fn config_key(&self, key: &str) -> Result<Widget<'a>> {
    let mut widget = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_single_config(
      self.camera,
      key.as_ptr() as *const c_char,
      &mut widget,
      self.context
    ))?;

    Ok(Widget::new(widget))
  }

  /// Apply a full config object to the cmaera.
  /// The configuration must be of type Window
  pub fn set_all_config(&self, config: &Widget) -> Result<()> {
    if !matches!(config.widget_type()?, WidgetType::Window) {
      Err("Full config object must be of type Window")?;
    }

    try_gp_internal!(libgphoto2_sys::gp_camera_set_config(
      self.camera,
      config.inner,
      self.context
    ))?;

    Ok(())
  }

  /// Set a single config to the camera
  pub fn set_config(&self, config: &Widget) -> Result<()> {
    try_gp_internal!(libgphoto2_sys::gp_camera_set_single_config(
      self.camera,
      config.name()?.as_ptr() as *const c_char,
      config.inner,
      self.context
    ))?;

    Ok(())
  }
}
