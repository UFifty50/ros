#![allow(non_snake_case)]
#![no_std]

extern crate alloc;

pub mod amd_extended;
pub mod amx;
pub mod base;
pub mod brand;
pub mod cache;
pub mod extended;
pub mod features;
pub mod hybrid;
pub mod hypervisor;
pub mod legacyCache;
pub mod monitor;
pub mod pmu;
pub mod power;
pub mod sgx;
pub mod time;
pub mod tlb;
pub mod topology;
pub mod trace;
pub mod vendor;
pub mod xsave;

use amd_extended::AmdSevInfo;
use amx::{AmxTileInfo, AmxTmulInfo};
use brand::ProcessorBrand;
use cache::CacheDescriptors;
use extended::{ExtendedProcessorInfo, AddressSizeInfo};
use features::{FeatureInfo, ExtendedFeatures, ExtendedFeatures1, ExtendedFeatures2};
use hybrid::HybridInfo;
use hypervisor::{HypervisorInfo, KvmFeatures};
use legacyCache::LegacyCacheDescriptors;
use monitor::MonitorMwaitInfo;
use pmu::PMonInfo;
use power::ThermalPowerInfo;
use sgx::SgxInfo;
use time::{TscInfo, ProcessorFrequency};
use tlb::TlbDescriptors;
use topology::TopologyDescriptors;
use trace::ProcessorTraceInfo;
use vendor::VendorInfo;
use xsave::XSaveInfo;


pub struct CPUID;

impl CPUID {
    pub fn amdSevInfo() -> Option<AmdSevInfo> { AmdSevInfo::read() }
    pub fn amxTileInfo() -> Option<AmxTileInfo> { AmxTileInfo::read() }
    pub fn amxTmulInfo() -> Option<AmxTmulInfo> { AmxTmulInfo::read() }
    pub fn addressSizeInfo() -> Option<AddressSizeInfo> { AddressSizeInfo::read() }
    pub fn cacheDescriptors() -> Option<CacheDescriptors> { CacheDescriptors::read() }
    pub fn extendedFeatures() -> Option<ExtendedFeatures> { ExtendedFeatures::read() }
    pub fn extendedFeatures1() -> Option<ExtendedFeatures1> { ExtendedFeatures1::read() }
    pub fn extendedFeatures2() -> Option<ExtendedFeatures2> { ExtendedFeatures2::read() }
    pub fn extendedProcessorInfo() -> Option<ExtendedProcessorInfo> { ExtendedProcessorInfo::read() }
    pub fn featureInfo() -> FeatureInfo { FeatureInfo::read() }
    pub fn hybridInfo() -> Option<HybridInfo> { HybridInfo::read() }
    pub fn hypervisorInfo() -> Option<HypervisorInfo> { HypervisorInfo::read() }
    pub fn kvmFeatures() -> KvmFeatures { KvmFeatures::read() }
    pub fn legacyCacheDescriptors() -> LegacyCacheDescriptors { LegacyCacheDescriptors::read() }
    pub fn monitorMwaitInfo() -> Option<MonitorMwaitInfo> { MonitorMwaitInfo::read() }
    pub fn pmonInfo() -> Option<PMonInfo> { PMonInfo::read() }
    pub fn processorBrand() -> Option<ProcessorBrand> { ProcessorBrand::read() }
    pub fn processorFrequency() -> Option<ProcessorFrequency> { ProcessorFrequency::read() }
    pub fn processorTraceInfo() -> Option<ProcessorTraceInfo> { ProcessorTraceInfo::read() }
    pub fn sgxInfo() -> Option<SgxInfo> { SgxInfo::read() }
    pub fn thermalPowerInfo() -> Option<ThermalPowerInfo> { ThermalPowerInfo::read() }
    pub fn tlbDescriptors() -> Option<TlbDescriptors> { TlbDescriptors::read() }
    pub fn topologyDescriptors() -> Option<TopologyDescriptors> { TopologyDescriptors::read() }
    pub fn tscInfo() -> Option<TscInfo> { TscInfo::read() }
    pub fn vendorInfo() -> VendorInfo { VendorInfo::read() }
    pub fn xsaveInfo() -> Option<XSaveInfo> { XSaveInfo::read() }
}

#[cfg(test)]
mod tests {}
