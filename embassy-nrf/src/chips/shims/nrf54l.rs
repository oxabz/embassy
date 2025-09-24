
macro_rules! dma_shim {
        ($id:ident, $peri_mod:ident, $peri_id:ident) => {
            pub(crate) trait $id {
                #[doc = "RXD EasyDMA channel"]
                fn rxd(self) -> nrf_pac::$peri_mod::DmaRx;
                #[doc = "TXD EasyDMA channel"]
                fn txd(self) -> nrf_pac::$peri_mod::DmaTx;
            }

            impl $id for nrf_pac::$peri_mod::$peri_id {

                #[inline(always)]
                fn rxd(self) -> nrf_pac::$peri_mod::DmaRx {
                    self.dma().rx()
                }

                #[inline(always)]
                fn txd(self) -> nrf_pac::$peri_mod::DmaTx {
                    self.dma().tx()
                }
            }
        };
    }

dma_shim!(TwimDmaShim, twim, Twim);

/// Simple trait that makes the nrf54l twim PAC compatible with the Twim driver
pub(crate) trait TwimShim{
    fn tasks_startrx(self) -> nrf_pac::common::Reg<u32, nrf_pac::common::W>;

    fn tasks_starttx(self) -> nrf_pac::common::Reg<u32, nrf_pac::common::W>;
}

impl TwimShim for nrf_pac::twim::Twim{
    #[inline(always)]
    fn tasks_startrx(self) -> nrf_pac::common::Reg<u32, nrf_pac::common::W> {
        unsafe { self.tasks_dma().rx().start() }
    }
    #[inline(always)]
    fn tasks_starttx(self) -> nrf_pac::common::Reg<u32, nrf_pac::common::W> {
        unsafe { self.tasks_dma().tx().start() }
    }
}

pub(crate) trait TwimShortsShim {
    #[doc = "Shortcut between event LASTTX and task STARTRX"]
    fn set_lasttx_startrx(&mut self, val: bool);

    fn set_lastrx_starttx(&mut self, val: bool);
}

impl TwimShortsShim for nrf_pac::twim::regs::Shorts{
    fn set_lasttx_startrx(&mut self, val: bool) {
        self.set_lasttx_dma_rx_start(val)
    }

    fn set_lastrx_starttx(&mut self, val: bool) {
        self.set_lastrx_dma_tx_start(val)
    }
}

dma_shim!(TwisDmaShim, twis, Twis);
dma_shim!(SpimDmaShim, spim, Spim);
dma_shim!(SpisDmaShim, spis, Spis);
