use embassy_hal_internal::PeripheralType;

#[cfg(all(feature = "_ppi", feature = "_multi_domain"))]
compile_error!("Multi domains is not supported for PPI controllers as none existed at the time I added peripheral support");

/// Interface for a domain
pub trait Domain {
    #[cfg(feature = "_dppi")]
    const PPI: crate::pac::dppic::Dppic;
    #[cfg(feature = "_ppi")]
    const PPI: crate::pac::ppi::Ppi;
}

/// Trait for peripheral types that are associated to a singular domain
pub trait DomainSpecific {
    type Domain: Domain;
}

/* MCUDomain the central domain */

#[cfg(not(feature = "_multi_domain"))]
pub enum MCUDomain{}

#[cfg(all(feature = "_dppi", not(feature = "_multi_domain")))]
impl Domain for MCUDomain {
    const PPI: crate::pac::dppic::Dppic = crate::pac::dppic::DPPIC;
}

#[cfg(all(not(feature = "_dppi"), not(feature = "_multi_domain")))]
impl Domain for MCUDomain {
    const PPI: crate::pac::ppi::Ppi = crate::pac::PPI;
}

#[cfg(feature = "_multi_domain")]
pub use crate::chip::MCUDomain;

#[macro_export]
macro_rules! domain_definition {
    ($type:ident, $ppi: path) => {
        pub enum $type{}
        impl $crate::domain::Domain for $type {
            #[cfg(feature = "_dppi")]
            const PPI: $crate::pac::dppic::Dppic = $ppi;
        }
    };
}

#[cfg(not(feature = "_multi_domain"))]
/// Defining peripheral type.
#[macro_export]
macro_rules! peripherals {
    ($($(#[$cfg:meta])? $name:ident),*$(,)?) => {
        embassy_hal_internal::peripherals_definition!(
            $(
                $(#[$cfg])?
                $name,
            )*
        );
        embassy_hal_internal::peripherals_struct!(
            $(
                $(#[$cfg])?
                $name,
            )*
        );

        $($(#[$cfg])?
        impl $crate::domain::DomainSpecific for $crate::peripherals::$name {
            type Domain = $crate::domain::MCUDomain;
        })*
    };
}

#[cfg(feature = "_multi_domain")]
/// Defining peripheral type.
#[macro_export]
macro_rules! peripherals {
    ($($domain:path => {$($(#[$cfg:meta])? $name:ident),*$(,)?})*) => {
        embassy_hal_internal::peripherals!(
            $($(
                $(#[$cfg])?
                $name,
            )*)*
        );

        $($(
        $(#[$cfg])?
        impl $crate::domain::DomainSpecific for $crate::peripherals::$name {
            type Domain = $domain;
        }
        )*)*
    };
}

