//! # Resource definitions for model weights, vocabularies and configuration files
//!
//! This crate relies on the concept of Resources to access the data used by the models.
//! This includes:
//! - model weights
//! - configuration files
//! - vocabularies
//! - (optional) merges files for BPE-based tokenizers
//!
//! These are expected in the pipelines configurations or are used as utilities to reference to the
//! resource location. Two types of resources are pre-defined:
//! - LocalResource: points to a local file
//! - RemoteResource: points to a remote file via a URL
//! - BufferResource: refers to a buffer that contains file contents for a resource (currently only
//!                   usable for weights)
//!
//! For `LocalResource` and `RemoteResource`, the local location of the file can be retrieved using
//! `get_local_path`, allowing to reference the resource file location regardless if it is a remote
//! or local resource. Default implementations for a number of `RemoteResources` are available as
//! pre-trained models in each model module.

mod buffer;
mod local;

use crate::common::error::RustBertError;
pub use buffer::BufferResource;
pub use local::LocalResource;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::RwLockWriteGuard;
use tch::nn::VarStore;

pub enum Resource<'a> {
    PathBuf(PathBuf),
    Buffer(RwLockWriteGuard<'a, Vec<u8>>),
}

/// # Resource Trait that can provide the location or data for the model, and location of
/// configuration or vocabulary resources
pub trait ResourceProvider: Debug + Send + Sync {
    /// Provides the local path for a resource.
    ///
    /// # Returns
    ///
    /// * `PathBuf` pointing to the resource file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rust_bert::resources::{LocalResource, ResourceProvider};
    /// use std::path::PathBuf;
    /// let config_resource = LocalResource {
    ///     local_path: PathBuf::from("path/to/config.json"),
    /// };
    /// let config_path = config_resource.get_local_path();
    /// ```
    fn get_local_path(&self) -> Result<PathBuf, RustBertError>;

    /// Provides access to an underlying resource.
    ///
    /// # Returns
    ///
    /// * `Resource` wrapping a representation of a resource.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rust_bert::resources::{BufferResource, LocalResource, ResourceProvider};
    /// ```
    fn get_resource(&self) -> Result<Resource, RustBertError>;
}

impl<T: ResourceProvider + ?Sized> ResourceProvider for Box<T> {
    fn get_local_path(&self) -> Result<PathBuf, RustBertError> {
        T::get_local_path(self)
    }
    fn get_resource(&self) -> Result<Resource, RustBertError> {
        T::get_resource(self)
    }
}

/// Load the provided `VarStore` with model weights from the provided `ResourceProvider`
pub fn load_weights(
    rp: &(impl ResourceProvider + ?Sized),
    vs: &mut VarStore,
) -> Result<(), RustBertError> {

    println!("----++++----++++ load_weights RP: ***** {:?}", rp);
    println!("----++++----++++ load_weights VS: ***** {:?}", vs);
    let resource = rp.get_resource();

    match resource {
        Ok(inner_res) => {
            match inner_res {
                Resource::Buffer(mut data) => {
                    println!("----++++----++++ load_weights **** 111111 ");
                    let load_res = vs.load_from_stream(std::io::Cursor::new(data.deref_mut()));
                    match load_res {
                        Ok(_) => {
                            println!("----++++----++++ load_weights ****---- 111111 ");
                            Ok(())
                        },
                        Err(e) => {
                            println!("----++++----++++ load_weights err2: ***** {:?}", e);
                            Err(e.into())
                        }
                    }
                }
                Resource::PathBuf(path) => {
                    println!("----++++----++++ load_weights **** 222222 ");
                    println!("----++++----++++ load_weights **** 222222 path:: {} ", path.clone());
                    let load_res = vs.load(path);
                    match load_res {
                        
                        Ok(inner_res) => {
                            println!("----++++----++++ load_weights **** ---- 222222 ");
                            Ok(inner_res)
                        },
                        Err(e) => {
                            println!("----++++----++++ load_weights err3: ***** {:?}", e);
                            Err(e.into())
                        }
                    }
                },
            }
        },
        Err(e) => {
            println!("----++++----++++ load_weights err1: ***** {:?}", e);
            Err(e)
        }
    }
}

#[cfg(feature = "remote")]
mod remote;
#[cfg(feature = "remote")]
pub use remote::RemoteResource;
