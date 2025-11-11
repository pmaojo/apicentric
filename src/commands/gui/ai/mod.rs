//! AI Service Generation Module
//!
//! This module handles AI-powered service definition generation for the GUI.

#![cfg(feature = "gui")]

pub mod generator;

pub use generator::AiServiceGenerator;
