// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! AArch64 specific tests.

#![cfg(target_arch = "aarch64")]

use crate::prelude::*;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
use tmk_core::aarch64;

#[tmk_test]
fn virtual_timer_irq(t: TestContext<'_>) {
    let timer_fired = AtomicBool::new(false);
    let timer_isr = |ctx: &mut aarch64::IrqContext| {
        if ctx.intid == aarch64::VIRTUAL_TIMER_PPI {
            aarch64::disable_virtual_timer();
            timer_fired.store(true, Relaxed);
        }
    };

    t.scope.subscope(|s| {
        s.set_irq_handler(&timer_isr);
        s.enable_gic_irq(aarch64::VIRTUAL_TIMER_PPI);

        let frequency = aarch64::read_cntfrq();
        let start = aarch64::read_cntvct();
        let ticks_until_interrupt = core::cmp::max(frequency / 100, 1);
        let timeout_ticks = core::cmp::max(frequency, ticks_until_interrupt * 10);

        aarch64::set_virtual_timer_compare(start + ticks_until_interrupt);
        s.enable_interrupts();

        while !timer_fired.load(Relaxed)
            && aarch64::read_cntvct().wrapping_sub(start) < timeout_ticks
        {
            aarch64::poll_interrupts();
            core::hint::spin_loop();
        }

        s.disable_interrupts();
        aarch64::disable_virtual_timer();
        s.disable_gic_irq(aarch64::VIRTUAL_TIMER_PPI);
    });

    assert!(
        timer_fired.load(Relaxed),
        "virtual timer interrupt did not fire"
    );
}
