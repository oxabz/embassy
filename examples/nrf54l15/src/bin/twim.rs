//! Example on how to read a Sensirion SHT4x i2c sensor.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]

use cortex_m::peripheral;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Frequency, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_nrf::pac::nfct::regs::Padconfig;
use embassy_nrf::pac::regulators::regs::Dcdcen;
use embassy_time::{Duration, Timer};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x52;

bind_interrupts!(struct Irqs {
    SERIAL30 => twim::InterruptHandler<peripherals::TWIM30>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    embassy_nrf::pac::REGULATORS.vregmain().dcdcen().write_value(Dcdcen(1));
    info!("{}", embassy_nrf::pac::REGULATORS.vregmain().inductordet().read().0);
    embassy_nrf::pac::NFCT.padconfig().write_value(Padconfig(0));

    Timer::after_millis(10).await;

    info!("Initializing TWI...");
    let mut config = twim::Config::default();
    config.scl_pullup = true;
    config.sda_pullup = true;
    config.frequency = Frequency::K100;
    config.sda_high_drive = false;
    config.scl_high_drive = false;
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);

    let spu_perm = embassy_nrf::pac::SPU20.periph(8).perm().read();
    info!("TWIM spu : ");
    info!(" - SECUREMAPPING : {}", spu_perm.securemapping().to_bits());
    info!(" - DMA : {}", spu_perm.dma().to_bits());
    info!(" - SECATTR : {}", spu_perm.secattr());
    info!(" - DMASEC : {}", spu_perm.dmasec());
    info!(" - LOCK : {}", spu_perm.lock());
    info!(" - PRESENT : {}", spu_perm.present().to_bits() != 0);

    let mut twi = Twim::new(p.TWIM30, Irqs, p.P0_00, p.P0_01, config, RAM_BUFFER.take());

    info!("Starting measurement...");
    info!("enable  = {:08x}",embassy_nrf::pac::TWIM30.enable().read().0);

    let mut buf = [0u8, 10];
    let res = twi.blocking_write_timeout(ADDRESS, &buf, Duration::from_secs(1));
    info!("Starting measurement...");

    info!("{}",embassy_nrf::pac::SPU30.events_periphaccerr().read());
    info!("dma     = {:08x}",embassy_nrf::pac::TWIM30.dma().tx().ptr().read());
    info!("amount  = {:08x}",embassy_nrf::pac::TWIM30.dma().tx().amount().read().0);
    info!("maxcnt  = {:08x}",embassy_nrf::pac::TWIM30.dma().tx().maxcnt().read().0);
    info!("buserro = {:08x}",embassy_nrf::pac::TWIM30.events_dma().tx().buserror().read());
    info!("end     = {:08x}",embassy_nrf::pac::TWIM30.events_dma().tx().end().read());
    info!("ready   = {:08x}",embassy_nrf::pac::TWIM30.events_dma().tx().ready().read());
    info!("stopped = {:08x}",embassy_nrf::pac::TWIM30.events_stopped().read());
    info!("error   = {:08x}",embassy_nrf::pac::TWIM30.events_error().read());
    info!("suspend = {:08x}",embassy_nrf::pac::TWIM30.events_suspended().read());
    info!("last_tx = {:08x}",embassy_nrf::pac::TWIM30.events_lasttx().read());
    info!("freq    = {:08x}",embassy_nrf::pac::TWIM30.frequency().read().0);
    info!("address = {:08x}",embassy_nrf::pac::TWIM30.address().read().0);
    info!("enable  = {:08x}",embassy_nrf::pac::TWIM30.enable().read().0);
    info!("scl     = {:08x}",embassy_nrf::pac::TWIM30.psel().scl().read().0);
    info!("sda     = {:08x}",embassy_nrf::pac::TWIM30.psel().sda().read().0);
    info!("clk     = {:08x}",embassy_nrf::pac::CLOCK.lfclk().stat().read().0);

    info!("Starting measurement...");

    Timer::after_millis(10).await;

    info!("Reading measurement");
    unwrap!(twi.read(ADDRESS, &mut buf).await);
    info!("Read: {=[u8]:x}", buf);

}
