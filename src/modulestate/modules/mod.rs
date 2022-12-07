mod aaa;
mod aab;
mod aap;
mod aas;
mod ccs;
mod css;
mod pac;
mod pal;
mod pcs;
mod ppo;
mod ppr;

use crate::mainboardstate::error::MainboardError;

pub fn get_module_validator(
    module_type: &str,
) -> Result<Box<dyn super::interface::ModuleValueValidator>, MainboardError> {
    if module_type == "AAA" {
        return Ok(Box::new(aaa::AAAValidator::new()));
    } else if module_type == "AAS" {
        return Ok(Box::new(aas::AASValidator::new()));
    } else if module_type == "AAP" {
        return Ok(Box::new(aap::AAPValidator::new()));
    } else if module_type == "AAB" {
        return Ok(Box::new(aab::AABValidator::new()));
    } else if module_type == "PAC" {
        return Ok(Box::new(pac::PACValidator::new()));
    } else if module_type == "PPO" {
        return Ok(Box::new(ppo::PPOValidator::new()));
    } else if module_type == "PPR" {
        return Ok(Box::new(ppr::PPRValidator::new()));
    } else if module_type == "PAL" {
        return Ok(Box::new(pal::PALValidator::new()));
    } else if module_type == "PCS" {
        return Ok(Box::new(pcs::PCSValidator::new()));
    } else if module_type == "CCS" {
        return Ok(Box::new(ccs::CCSValidator::new()));
    } else if module_type == "CSS" {
        return Ok(Box::new(css::CSSValidator::new()));
    } else {
        return Err(
            MainboardError::new().message("cannot find validator for module type".to_string())
        );
    }
}
