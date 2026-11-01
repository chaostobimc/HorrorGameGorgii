#!/usr/bin/env python3
"""
Procedural sound design for "The Diplomat Peak".

Everything in assets/audio is synthesised here: tropical rain on glass,
thunder rolling through the Cordillera, dry wood creaks, the freight
elevator, breaker clunks, a detuned ballroom piano, and the final stinger.

Run with:  python3 tools/gen_audio.py
Requires:  numpy
"""

import math
import os
import wave

import numpy as np

OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "audio")
os.makedirs(OUT, exist_ok=True)
SR = 44100
rng = np.random.default_rng(1988)


# ----------------------------------------------------------------------------
# helpers
# ----------------------------------------------------------------------------


def t_axis(dur, sr=SR):
    return np.arange(int(dur * sr), dtype=np.float32) / sr


def normalize(x, peak=0.9):
    m = float(np.max(np.abs(x))) or 1.0
    return (x / m * peak).astype(np.float32)


def fade(x, fade_in=0.002, fade_out=0.01, sr=SR):
    n = len(x)
    a = int(fade_in * sr)
    b = int(fade_out * sr)
    if a > 0:
        x[:a] *= np.linspace(0, 1, a) ** 0.5
    if b > 0:
        x[-b:] *= np.linspace(1, 0, b) ** 0.8
    return x


def spectral_noise(n, alpha=1.0, seed=0, sr=SR):
    """1/f^alpha noise (alpha 0 = white, 1 = pink, 2 = brown)."""
    r = np.random.default_rng(seed)
    spec = np.fft.rfft(r.normal(size=n).astype(np.float64))
    f = np.fft.rfftfreq(n, 1.0 / sr)
    f[0] = 1.0
    spec /= f ** (alpha / 2.0)
    out = np.fft.irfft(spec, n).astype(np.float32)
    return out


def band(x, lo=None, hi=None, slope_lo=2.0, slope_hi=2.0, sr=SR):
    """Smooth FFT band limiting."""
    n = len(x)
    spec = np.fft.rfft(x)
    f = np.fft.rfftfreq(n, 1.0 / sr)
    f[0] = 1e-3
    g = np.ones_like(f)
    if lo:
        g *= 1.0 / (1.0 + (lo / f) ** slope_lo)
    if hi:
        g *= 1.0 / (1.0 + (f / hi) ** slope_hi)
    return np.fft.irfft(spec * g, n).astype(np.float32)


def resonance(x, freq, q=12.0, sr=SR):
    """Narrow band emphasis around freq (FFT gaussian bump)."""
    n = len(x)
    spec = np.fft.rfft(x)
    f = np.fft.rfftfreq(n, 1.0 / sr)
    g = 1.0 + 14.0 * np.exp(-0.5 * ((f - freq) / (freq / q)) ** 2)
    return np.fft.irfft(spec * g, n).astype(np.float32)


def reverb(x, decay=1.2, wet=0.35, seed=7, sr=SR, pre=0.02):
    """Convolution reverb with a synthetic noise tail."""
    n = int(decay * sr)
    ir = np.random.default_rng(seed).normal(size=n).astype(np.float32)
    ir *= np.exp(-np.linspace(0, 6.0, n))
    ir = band(ir, lo=200, hi=6000)
    ir = normalize(ir, 1.0)
    pad = np.zeros(int(pre * sr), np.float32)
    ir = np.concatenate([pad, ir])
    wet_sig = np.fft.irfft(np.fft.rfft(x, len(x) + len(ir)) * np.fft.rfft(ir, len(x) + len(ir)))
    wet_sig = wet_sig[: len(x)]
    wet_sig = normalize(wet_sig, 1.0)
    return (x * (1.0 - wet) + wet_sig * wet).astype(np.float32)


def env_ad(n, attack, decay, sr=SR):
    a = max(1, int(attack * sr))
    e = np.ones(n, np.float32)
    e[:a] = np.linspace(0, 1, a) ** 0.7
    tail = np.exp(-np.linspace(0, 8.0, n - a))
    e[a:] = tail
    return e


def sine(freq, dur, sr=SR, phase=0.0):
    return np.sin(2 * np.pi * freq * t_axis(dur, sr) + phase).astype(np.float32)


def sweep(f0, f1, dur, sr=SR, curve=1.0):
    t = t_axis(dur, sr)
    f = f0 + (f1 - f0) * (t / dur) ** curve
    phase = 2 * np.pi * np.cumsum(f) / sr
    return np.sin(phase).astype(np.float32)


def save(name, data, sr=SR, stereo=False):
    d = np.asarray(data, np.float32)
    if d.ndim == 1:
        d = d[:, None]
    peak = float(np.max(np.abs(d))) or 1.0
    d = d / peak * 0.92
    pcm = (np.clip(d, -1, 1) * 32767).astype(np.int16)
    path = os.path.join(OUT, name + ".wav")
    with wave.open(path, "wb") as w:
        w.setnchannels(d.shape[1])
        w.setsampwidth(2)
        w.setframerate(sr)
        w.writeframes(pcm.tobytes())
    print("  ->", name + ".wav", f"{os.path.getsize(path)/1024:.0f} KB")


def loopify(x, cross=0.35, sr=SR):
    """Crossfade the tail into the head so the loop is seamless."""
    n = int(cross * sr)
    if n * 2 >= len(x):
        return x
    head = x[:n].copy()
    tail = x[-n:].copy()
    ramp = np.linspace(0, 1, n, dtype=np.float32)[:, None] if x.ndim == 2 else np.linspace(0, 1, n, dtype=np.float32)
    x = x[:-n]
    x[:n] = x[:n] * ramp + tail * (1 - ramp)
    return x


# ----------------------------------------------------------------------------
# ambience loops
# ----------------------------------------------------------------------------


def rain_loop(dur=24.0, sr=32000, seed=1):
    n = int(dur * sr)
    left = spectral_noise(n, alpha=0.75, seed=seed, sr=sr)
    right = spectral_noise(n, alpha=0.75, seed=seed + 1, sr=sr)
    left = band(left, lo=260, hi=7000, sr=sr)
    right = band(right, lo=260, hi=7000, sr=sr)

    # slow intensity gusts (squalls crossing the mountain)
    tt = np.arange(n) / sr
    gust = (
        0.72
        + 0.16 * np.sin(2 * np.pi * tt / 11.0 + 0.5)
        + 0.10 * np.sin(2 * np.pi * tt / 5.3 + 2.1)
        + 0.06 * np.sin(2 * np.pi * tt / 2.7)
    ).astype(np.float32)
    left *= gust
    right *= gust

    # individual droplets on glass / roof
    drops = np.zeros((n, 2), np.float32)
    for _ in range(int(dur * 42)):
        pos = int(rng.integers(0, n - 3000))
        f = float(rng.uniform(500, 2600))
        ln = int(rng.integers(260, 1400))
        e = np.exp(-np.linspace(0, 7.0, ln)).astype(np.float32)
        d = np.sin(2 * np.pi * f * np.arange(ln) / sr) * e * float(rng.uniform(0.03, 0.14))
        ch = int(rng.integers(0, 2))
        drops[pos : pos + ln, ch] += d

    # low rumble of water on the roof structure
    rumble = band(spectral_noise(n, alpha=2.0, seed=seed + 9, sr=sr), hi=140, sr=sr)
    rumble = normalize(rumble, 0.28)
    mix = np.stack([left, right], 1) * 0.62 + drops + rumble[:, None]
    mix = loopify(normalize(mix, 0.85), cross=1.2, sr=sr)
    save("rain_loop", mix, sr=sr, stereo=True)


def wind_loop(dur=22.0, sr=32000, seed=5):
    n = int(dur * sr)
    base = spectral_noise(n, alpha=1.9, seed=seed, sr=sr)
    base = band(base, lo=45, hi=520, sr=sr)
    # whistle through broken window frames
    whistle_f = 520.0 + 180.0 * np.sin(2 * np.pi * np.arange(n) / sr / 9.0)
    tt = np.arange(n) / sr
    whistle = np.sin(2 * np.pi * np.cumsum(whistle_f) / sr).astype(np.float32)
    whistle *= band(spectral_noise(n, alpha=1.0, seed=seed + 3, sr=sr), hi=8, sr=sr) * 0.5
    gust = (0.55 + 0.45 * np.sin(2 * np.pi * tt / 7.5 + 1.0) ** 2).astype(np.float32)
    left = base * gust + whistle * 0.25 * gust
    right = np.roll(base, 1371) * np.roll(gust, 331) + whistle * 0.18
    mix = loopify(normalize(np.stack([left, right], 1), 0.8), cross=1.5, sr=sr)
    save("wind_loop", mix, sr=sr, stereo=True)


def tone_loop(dur=18.0, seed=11):
    """Dead-air room tone: 50 Hz mains hum bleeding through the walls."""
    n = int(dur * SR)
    t = np.arange(n) / SR
    hum = (
        np.sin(2 * np.pi * 50 * t) * 0.30
        + np.sin(2 * np.pi * 100 * t + 1.1) * 0.16
        + np.sin(2 * np.pi * 150 * t + 2.3) * 0.06
        + np.sin(2 * np.pi * 25 * t) * 0.12
    ).astype(np.float32)
    air = band(spectral_noise(n, alpha=1.6, seed=seed), lo=60, hi=1200)
    mix = normalize(hum * 0.55 + normalize(air, 0.5) * 0.45, 0.55)
    mix = loopify(mix, cross=0.8)
    save("tone_loop", mix)


def gen_hum_loop(dur=9.0, seed=13):
    n = int(dur * SR)
    t = np.arange(n) / SR
    hum = (
        np.sin(2 * np.pi * 60 * t) * 0.4
        + np.sin(2 * np.pi * 120 * t + 0.4) * 0.22
        + np.sin(2 * np.pi * 180 * t + 1.7) * 0.10
        + np.sin(2 * np.pi * 240 * t + 0.9) * 0.05
    ).astype(np.float32)
    mech = band(spectral_noise(n, alpha=1.2, seed=seed), lo=200, hi=2600) * 0.35
    wobble = (0.85 + 0.15 * np.sin(2 * np.pi * t / 1.7)).astype(np.float32)
    mix = normalize((hum + mech) * wobble, 0.75)
    mix = loopify(mix, cross=0.6)
    save("gen_hum_loop", mix)


def elev_loop(dur=8.0, seed=17):
    n = int(dur * SR)
    t = np.arange(n) / SR
    rumble = band(spectral_noise(n, alpha=2.0, seed=seed), hi=90) * 0.9
    grind = band(spectral_noise(n, alpha=0.9, seed=seed + 2), lo=350, hi=1800) * 0.18
    cable = np.sin(2 * np.pi * 41 * t).astype(np.float32) * 0.15
    clatter = np.zeros(n, np.float32)
    for _ in range(26):
        p = int(rng.integers(0, n - 2000))
        ln = int(rng.integers(200, 1400))
        clatter[p : p + ln] += np.exp(-np.linspace(0, 9, ln)).astype(np.float32) * float(rng.uniform(0.05, 0.2))
    mix = normalize(rumble + grind + cable + clatter, 0.7)
    mix = loopify(mix, cross=0.5)
    save("elev_loop", mix)


def buzz_loop(dur=6.0, seed=19):
    n = int(dur * SR)
    t = np.arange(n) / SR
    buzz = (
        np.sign(np.sin(2 * np.pi * 120 * t)).astype(np.float32) * 0.12
        + np.sin(2 * np.pi * 240 * t) * 0.05
    )
    crackle = np.zeros(n, np.float32)
    for _ in range(int(dur * 26)):
        p = int(rng.integers(0, n - 900))
        ln = int(rng.integers(60, 500))
        crackle[p : p + ln] += rng.normal(size=ln).astype(np.float32) * float(rng.uniform(0.05, 0.3)) * np.exp(
            -np.linspace(0, 6, ln)
        )
    hiss = band(spectral_noise(n, alpha=0.6, seed=seed), lo=900, hi=9000) * 0.12
    mix = normalize(buzz + crackle + hiss, 0.5)
    mix = loopify(mix, cross=0.4)
    save("buzz_loop", mix)


# ----------------------------------------------------------------------------
# weather one-shots
# ----------------------------------------------------------------------------


def thunder(name, dur, seed, near=False):
    n = int(dur * SR)
    t = np.arange(n) / SR
    body = spectral_noise(n, alpha=2.0, seed=seed)
    body = band(body, lo=28, hi=260 if not near else 420)
    # rolling waves of rumble
    roll = np.zeros(n, np.float32)
    for k in range(4):
        f = float(rng.uniform(0.25, 0.8))
        ph = float(rng.uniform(0, 6.28))
        roll += np.sin(2 * np.pi * f * t + ph) * float(rng.uniform(0.3, 0.7))
    roll = (1.2 + 0.55 * roll).astype(np.float32)
    env = np.exp(-np.linspace(0, 4.2, n)).astype(np.float32)
    env[: int(0.03 * SR)] = np.linspace(0, 1, int(0.03 * SR))
    sig = body * roll * env
    if near:
        crack = rng.normal(size=n).astype(np.float32)
        crack = band(crack, lo=120, hi=6500)
        crack_env = np.exp(-np.linspace(0, 9.0, n)).astype(np.float32)
        crack_env[: int(0.004 * SR)] = np.linspace(0, 1, int(0.004 * SR))
        sig += crack * crack_env * 0.30
        # a hard slap right at the front
        slap = np.exp(-np.linspace(0, 22.0, n)).astype(np.float32) * np.sin(2 * np.pi * 90 * t)
        sig += slap * 0.5
    sig = band(sig, hi=1400 if not near else 2200, slope_hi=1.4)
    sig = reverb(normalize(sig, 0.8), decay=2.5 if not near else 3.2, wet=0.4, seed=seed + 1)
    sig = band(sig, hi=2600 if near else 1600, slope_hi=1.6)
    save(name, fade(normalize(sig, 0.9), 0.001, 0.4))


def thunder_set():
    thunder("thunder_far_1", 6.5, 31)
    thunder("thunder_far_2", 7.5, 33)
    thunder("thunder_far_3", 5.5, 35)
    thunder("thunder_near_1", 5.0, 37, near=True)
    thunder("thunder_near_2", 6.0, 39, near=True)


# ----------------------------------------------------------------------------
# water / building
# ----------------------------------------------------------------------------


def drip(name, seed, f0=1500.0, dur=0.55):
    n = int(dur * SR)
    t = np.arange(n) / SR
    f = f0 * np.exp(-t * 9.0) + 420.0
    body = np.sin(2 * np.pi * np.cumsum(f) / SR).astype(np.float32)
    body *= np.exp(-np.linspace(0, 12.0, n)).astype(np.float32)
    click = rng.normal(size=n).astype(np.float32) * np.exp(-np.linspace(0, 60.0, n))
    click = band(click, lo=1500, hi=9000)
    sig = body * 0.7 + click * 0.25
    sig = reverb(sig, decay=0.8, wet=0.45, seed=seed)
    save(name, fade(normalize(sig, 0.8), 0.001, 0.05))


def drip_set():
    drip("drip_1", 41, 1500)
    drip("drip_2", 43, 1150)
    drip("drip_3", 45, 1900, dur=0.7)


def creak(name, seed, dur=1.1, f_lo=180.0, f_hi=900.0):
    """Stick-slip wood creak: resonant noise with a wandering centre frequency."""
    n = int(dur * SR)
    steps = 12
    sig = np.zeros(n, np.float32)
    t = np.arange(n) / SR
    wobble = (
        0.5 * np.sin(2 * np.pi * 3.1 * t + rng.uniform(0, 6))
        + 0.3 * np.sin(2 * np.pi * 5.7 * t + rng.uniform(0, 6))
        + 0.2 * np.sin(2 * np.pi * 1.3 * t)
    )
    centre = f_lo + (f_hi - f_lo) * (t / dur) + 90.0 * wobble
    # simple time-varying resonator: modulated ring per grain
    base = spectral_noise(n, alpha=1.2, seed=seed)
    base = band(base, lo=f_lo * 0.7, hi=f_hi * 1.4)
    amp = np.abs(wobble) ** 1.6
    amp *= np.exp(-np.linspace(0, 2.6, n))
    sig = base * amp
    # metallic wood "grain" clicks
    for _ in range(int(dur * 60)):
        p = int(rng.integers(0, n - 400))
        ln = int(rng.integers(40, 380))
        sig[p : p + ln] += (
            band(rng.normal(size=ln).astype(np.float32), hi=2500)
            * np.exp(-np.linspace(0, 8, ln))
            * float(rng.uniform(0.02, 0.09))
        )
    sig = resonance(sig, float(np.mean(centre)), q=18.0)
    sig = band(sig, lo=90, hi=2400, slope_hi=1.8)
    sig = reverb(normalize(sig, 0.8), decay=1.0, wet=0.3, seed=seed + 1)
    save(name, fade(normalize(sig, 0.75), 0.004, 0.1))


def creak_set():
    creak("creak_1", 51, 1.4, 150, 700)
    creak("creak_2", 53, 0.9, 220, 1100)
    creak("creak_3", 55, 1.8, 120, 520)
    creak("creak_4", 57, 0.7, 300, 1400)


def step_wood(name, seed, weight=1.0):
    n = int(0.38 * SR)
    patch = np.zeros(n, np.float32)
    # heel thud
    f = 70.0
    thud = np.sin(2 * np.pi * f * np.arange(n) / SR).astype(np.float32) * np.exp(
        -np.linspace(0, 26, n)
    )
    # wood body resonance
    body = rng.normal(size=n).astype(np.float32)
    body = band(body, lo=90, hi=1400) * np.exp(-np.linspace(0, 16, n))
    body = resonance(body, 320.0, q=7.0)
    # dust / grit
    grit = rng.normal(size=n).astype(np.float32) * np.exp(-np.linspace(0, 55, n))
    grit = band(grit, lo=1500, hi=6500) * 0.16
    sig = thud * 0.5 + body * 0.5 + grit
    sig = reverb(normalize(sig, 0.8) * weight, decay=0.5, wet=0.22, seed=seed)
    save(name, fade(normalize(sig, 0.7), 0.001, 0.06))


def step_stone(name, seed):
    n = int(0.34 * SR)
    body = rng.normal(size=n).astype(np.float32)
    body = band(body, lo=200, hi=4200) * np.exp(-np.linspace(0, 22, n))
    body = resonance(body, 1400.0, q=5.0)
    thud = np.sin(2 * np.pi * 95 * np.arange(n) / SR).astype(np.float32) * np.exp(-np.linspace(0, 34, n))
    wet = rng.normal(size=n).astype(np.float32) * np.exp(-np.linspace(0, 12, n))
    wet = band(wet, lo=2500, hi=10000) * 0.2
    sig = normalize(body * 0.55 + thud * 0.45 + wet, 0.8)
    sig = reverb(sig, decay=0.9, wet=0.35, seed=seed)
    save(name, fade(normalize(sig, 0.7), 0.001, 0.06))


def steps_set():
    for i, s in enumerate((61, 63, 65, 67)):
        step_wood(f"step_wood_{i+1}", s, weight=1.0 if i else 0.9)
    for i, s in enumerate((71, 73, 75)):
        step_stone(f"step_stone_{i+1}", s)


def door_slam():
    dur = 1.9
    n = int(dur * SR)
    t = np.arange(n) / SR
    thud = np.sin(2 * np.pi * 48 * t).astype(np.float32) * np.exp(-np.linspace(0, 16, n))
    wood = rng.normal(size=n).astype(np.float32)
    wood = band(wood, lo=80, hi=2200) * np.exp(-np.linspace(0, 11, n))
    wood = resonance(wood, 180.0, q=9.0)
    # latch rattle
    rat = np.zeros(n, np.float32)
    for k in range(7):
        p = int(rng.integers(0, int(0.25 * SR)))
        ln = int(rng.integers(200, 900))
        rat[p : p + ln] += (
            rng.normal(size=ln).astype(np.float32)
            * np.exp(-np.linspace(0, 9, ln))
            * float(rng.uniform(0.05, 0.18))
        )
    rat = resonance(rat, 2600.0, q=14.0)
    sig = normalize(thud * 0.8 + wood * 0.55 + rat * 0.3, 0.9)
    sig = reverb(sig, decay=2.2, wet=0.42, seed=77)
    save("door_slam", fade(normalize(sig, 0.95), 0.001, 0.3))


def door_creak():
    dur = 2.6
    n = int(dur * SR)
    t = np.arange(n) / SR
    prog = np.clip(t / dur, 0, 1)
    centre = 240.0 + 520.0 * np.sin(prog * np.pi) + 60.0 * np.sin(2 * np.pi * 6.0 * t)
    base = spectral_noise(n, alpha=1.1, seed=81)
    base = band(base, lo=150, hi=1600)
    amp = np.sin(prog * np.pi) ** 2
    pulses = (0.6 + 0.4 * np.sin(2 * np.pi * 11 * t) ** 2).astype(np.float32)
    sig = base * amp * pulses
    sig = resonance(sig, float(np.mean(centre)), q=16.0)
    sig = reverb(normalize(sig, 0.8), decay=1.6, wet=0.35, seed=83)
    save("door_creak", fade(normalize(sig, 0.8), 0.02, 0.2))


# ----------------------------------------------------------------------------
# mechanism / props
# ----------------------------------------------------------------------------


def clock_tick(name="clock_tick", seed=91):
    n = int(0.16 * SR)
    click = rng.normal(size=n).astype(np.float32) * np.exp(-np.linspace(0, 40, n))
    click = band(click, lo=1200, hi=9000)
    body = np.sin(2 * np.pi * 2400 * np.arange(n) / SR).astype(np.float32) * np.exp(
        -np.linspace(0, 60, n)
    )
    sig = reverb(normalize(click * 0.8 + body * 0.35, 0.7), decay=0.35, wet=0.25, seed=seed)
    save(name, fade(normalize(sig, 0.6), 0.0005, 0.02))


def gear_clunk(name="gear_clunk", seed=93):
    dur = 1.6
    n = int(dur * SR)
    t = np.arange(n) / SR
    hits = np.zeros(n, np.float32)
    for k, off in enumerate((0.0, 0.13, 0.31)):
        p = int(off * SR)
        ln = min(n - p, int(1.2 * SR))
        tt = np.arange(ln) / SR
        ring = (
            np.sin(2 * np.pi * 320 * tt) * 0.6
            + np.sin(2 * np.pi * 517 * tt) * 0.4
            + np.sin(2 * np.pi * 811 * tt) * 0.25
        ).astype(np.float32)
        hits[p : p + ln] += ring * np.exp(-np.linspace(0, 9, ln)) * (1.0 - k * 0.25)
    thud = np.sin(2 * np.pi * 60 * t).astype(np.float32) * np.exp(-np.linspace(0, 18, n))
    noise = band(rng.normal(size=n).astype(np.float32), lo=200, hi=4000) * np.exp(
        -np.linspace(0, 12, n)
    )
    sig = normalize(hits * 0.7 + thud * 0.5 + noise * 0.35, 0.9)
    sig = reverb(sig, decay=1.8, wet=0.35, seed=seed)
    save(name, fade(normalize(sig, 0.85), 0.001, 0.25))


def beeps():
    # keypad button
    n = int(0.09 * SR)
    t = np.arange(n) / SR
    s = np.sin(2 * np.pi * 1520 * t).astype(np.float32) * np.exp(-np.linspace(0, 14, n))
    s += np.sin(2 * np.pi * 3040 * t).astype(np.float32) * 0.2 * np.exp(-np.linspace(0, 22, n))
    save("keypad_beep", fade(normalize(s, 0.45), 0.001, 0.02))

    # error: two low buzzes
    n = int(0.6 * SR)
    t = np.arange(n) / SR
    s = np.zeros(n, np.float32)
    for off, f in ((0.0, 320.0), (0.26, 240.0)):
        p = int(off * SR)
        ln = int(0.22 * SR)
        tt = np.arange(ln) / SR
        tone = (
            np.sign(np.sin(2 * np.pi * f * tt)).astype(np.float32) * 0.5
            + np.sin(2 * np.pi * f * 2 * tt) * 0.3
        )
        tone *= np.exp(-np.linspace(0, 5, ln))
        s[p : p + ln] += tone
    s = band(s, hi=4500)
    save("keypad_error", fade(normalize(s, 0.5), 0.002, 0.05))

    # success: rising third
    n = int(0.9 * SR)
    s = np.zeros(n, np.float32)
    for off, f in ((0.0, 880.0), (0.16, 1320.0), (0.32, 1760.0)):
        p = int(off * SR)
        ln = int(0.45 * SR)
        tt = np.arange(ln) / SR
        tone = np.sin(2 * np.pi * f * tt).astype(np.float32) * np.exp(-np.linspace(0, 6, ln))
        s[p : p + ln] += tone
    save("keypad_ok", fade(normalize(s, 0.5), 0.002, 0.12))


def lever_clunk():
    dur = 0.9
    n = int(dur * SR)
    t = np.arange(n) / SR
    metal = (
        np.sin(2 * np.pi * 640 * t) * 0.4
        + np.sin(2 * np.pi * 1180 * t) * 0.25
        + np.sin(2 * np.pi * 1730 * t) * 0.12
    ).astype(np.float32) * np.exp(-np.linspace(0, 11, n))
    thud = np.sin(2 * np.pi * 85 * t).astype(np.float32) * np.exp(-np.linspace(0, 20, n))
    noise = band(rng.normal(size=n).astype(np.float32), lo=400, hi=5200) * np.exp(
        -np.linspace(0, 16, n)
    )
    sig = reverb(normalize(metal * 0.6 + thud * 0.6 + noise * 0.4, 0.9), decay=1.1, wet=0.3, seed=101)
    save("lever_clunk", fade(normalize(sig, 0.8), 0.001, 0.2))


def spark():
    dur = 1.2
    n = int(dur * SR)
    s = np.zeros(n, np.float32)
    for _ in range(120):
        p = int(rng.integers(0, n - 1200))
        ln = int(rng.integers(30, 900))
        s[p : p + ln] += rng.normal(size=ln).astype(np.float32) * float(rng.uniform(0.05, 0.5)) * np.exp(
            -np.linspace(0, 6, ln)
        )
    s = band(s, lo=800, hi=13000)
    zap = np.sin(2 * np.pi * 90 * np.arange(n) / SR).astype(np.float32) * np.exp(-np.linspace(0, 9, n))
    sig = normalize(s * 0.8 + zap * 0.25, 0.9)
    sig = reverb(sig, decay=1.4, wet=0.3, seed=103)
    save("spark", fade(normalize(sig, 0.85), 0.001, 0.1))


def generator_start():
    dur = 3.6
    n = int(dur * SR)
    t = np.arange(n) / SR
    # engine catches, then settles into a hum
    crank = np.zeros(n, np.float32)
    for k in range(4):
        p = int((0.1 + k * 0.28) * SR)
        ln = int(0.2 * SR)
        tt = np.arange(ln) / SR
        crank[p : p + ln] += np.sin(2 * np.pi * (140 - k * 10) * tt).astype(np.float32) * np.exp(
            -np.linspace(0, 8, ln)
        )
    hum_f = 55.0 + 25.0 * np.exp(-t * 1.6)
    hum = np.sin(2 * np.pi * np.cumsum(hum_f) / SR).astype(np.float32)
    hum *= np.clip(t / 0.6, 0, 1)
    hum *= 0.8 + 0.2 * np.sin(2 * np.pi * 3 * t)
    rattle = band(spectral_noise(n, alpha=1.0, seed=107), lo=300, hi=3000) * 0.12 * np.clip(t / 1.0, 0, 1)
    sig = normalize(hum * 0.7 + crank * 0.5 + rattle, 0.9)
    save("gen_start", fade(normalize(sig, 0.85), 0.01, 0.5))


def piano_note(name="piano_note", root=146.83, seed=111):
    dur = 5.0
    n = int(dur * SR)
    t = np.arange(n) / SR
    sig = np.zeros(n, np.float32)
    for h in range(1, 13):
        det = 1.0 + 0.0009 * h * h  # slight inharmonicity
        amp = (1.0 / h**1.35) * (0.7 + 0.3 * rng.random())
        decay = 3.0 + h * 0.7
        sig += (
            np.sin(2 * np.pi * root * h * det * t + float(rng.uniform(0, 6))).astype(np.float32)
            * amp
            * np.exp(-t * decay)
        )
    # hammer noise
    ham = band(rng.normal(size=n).astype(np.float32), lo=900, hi=6000) * np.exp(-np.linspace(0, 70, n)) * 0.15
    sig = normalize(sig + ham, 0.8)
    sig = reverb(sig, decay=2.6, wet=0.45, seed=seed)
    save(name, fade(normalize(sig, 0.75), 0.001, 0.4))


def stinger():
    """0.4 s of pure panic: FM screech + noise burst + sub drop. No gore, all shock."""
    dur = 0.42
    n = int(dur * SR)
    t = np.arange(n) / SR
    # FM screech
    carrier = 1800.0 + 2200.0 * np.exp(-t * 9.0)
    mod = np.sin(2 * np.pi * 137.0 * t) * (9.0 * np.exp(-t * 5.0))
    screech = np.sin(2 * np.pi * np.cumsum(carrier) / SR + mod).astype(np.float32)
    # broadband burst
    burst = rng.normal(size=n).astype(np.float32)
    burst = band(burst, lo=300, hi=15000)
    # sub impact
    sub = np.sin(2 * np.pi * (90 - 60 * t / dur) * t).astype(np.float32)
    env = np.exp(-np.linspace(0, 7.0, n)).astype(np.float32)
    env[: int(0.002 * SR)] = np.linspace(0, 1, int(0.002 * SR))
    sig = normalize(np.tanh((screech * 0.55 + burst * 0.5 + sub * 0.4) * 2.2), 0.98) * env
    sig = reverb(normalize(sig, 0.95), decay=1.2, wet=0.22, seed=121)
    save("stinger", fade(normalize(sig, 0.99), 0.0005, 0.03))


def heartbeat():
    dur = 1.5
    n = int(dur * SR)
    t = np.arange(n) / SR
    sig = np.zeros(n, np.float32)
    for off, amp in ((0.0, 1.0), (0.30, 0.72)):
        p = int(off * SR)
        ln = int(0.45 * SR)
        tt = np.arange(ln) / SR
        beat = np.sin(2 * np.pi * 48 * tt).astype(np.float32) * np.exp(-np.linspace(0, 18, ln))
        beat += band(rng.normal(size=ln).astype(np.float32), hi=260) * np.exp(-np.linspace(0, 26, ln)) * 0.5
        sig[p : p + ln] += beat * amp
    sig = reverb(normalize(sig, 0.8), decay=0.9, wet=0.25, seed=131)
    save("heartbeat", fade(normalize(sig, 0.8), 0.004, 0.2))


def paper(name, dur, seed, bright=8000.0):
    n = int(dur * SR)
    s = np.zeros(n, np.float32)
    for _ in range(int(dur * 90)):
        p = int(rng.integers(0, max(1, n - 2000)))
        ln = int(rng.integers(300, 2000))
        ln = min(ln, n - p)
        s[p : p + ln] += (
            rng.normal(size=ln).astype(np.float32)
            * np.exp(-np.linspace(0, 7, ln))
            * float(rng.uniform(0.1, 0.5))
        )
    s = band(s, lo=1200, hi=bright)
    sig = reverb(normalize(s, 0.7), decay=0.4, wet=0.2, seed=seed)
    save(name, fade(normalize(sig, 0.55), 0.005, 0.05))


def whisper():
    dur = 3.2
    n = int(dur * SR)
    t = np.arange(n) / SR
    src = spectral_noise(n, alpha=0.9, seed=141)
    src = band(src, lo=300, hi=4200)
    # syllable-like amplitude modulation
    syl = np.zeros(n, np.float32)
    pos = 0.0
    while pos < dur:
        ln = float(rng.uniform(0.12, 0.32))
        gap = float(rng.uniform(0.05, 0.22))
        a = int(pos * SR)
        b = min(n, int((pos + ln) * SR))
        if b > a:
            syl[a:b] = np.hanning(b - a).astype(np.float32)
        pos += ln + gap
    breath = band(spectral_noise(n, alpha=1.4, seed=143), lo=200, hi=1500) * 0.25
    sig = (src + breath) * syl
    sig = resonance(sig, 900.0, q=6.0) + resonance(sig, 2400.0, q=9.0) * 0.6
    sig = reverb(normalize(sig, 0.6), decay=1.8, wet=0.45, seed=145)
    save("whisper", fade(normalize(sig, 0.7), 0.05, 0.3))


def distant_steps():
    dur = 3.0
    n = int(dur * SR)
    sig = np.zeros(n, np.float32)
    for k in range(4):
        p = int((0.1 + k * 0.42) * SR)
        ln = int(0.3 * SR)
        ln = min(ln, n - p)
        tt = np.arange(ln) / SR
        s = np.sin(2 * np.pi * 78 * tt).astype(np.float32) * np.exp(-np.linspace(0, 30, ln))
        s += band(rng.normal(size=ln).astype(np.float32), hi=700) * np.exp(-np.linspace(0, 30, ln)) * 0.4
        sig[p : p + ln] += s
    sig = reverb(normalize(sig, 0.7), decay=2.4, wet=0.6, seed=151)
    save("distant_steps", fade(normalize(sig, 0.6), 0.01, 0.2))


def switch_click():
    dur = 0.3
    n = int(dur * SR)
    s = rng.normal(size=n).astype(np.float32) * np.exp(-np.linspace(0, 45, n))
    s = band(s, lo=900, hi=11000)
    thud = np.sin(2 * np.pi * 160 * np.arange(n) / SR).astype(np.float32) * np.exp(-np.linspace(0, 55, n))
    sig = reverb(normalize(s * 0.7 + thud * 0.4, 0.7), decay=0.5, wet=0.25, seed=161)
    save("switch_click", fade(normalize(sig, 0.6), 0.0005, 0.03))


def led_on():
    dur = 1.2
    n = int(dur * SR)
    t = np.arange(n) / SR
    s = np.sin(2 * np.pi * 1200 * t).astype(np.float32) * np.clip(1 - t / 0.5, 0, 1) ** 2
    s += np.sin(2 * np.pi * 2400 * t).astype(np.float32) * 0.3 * np.clip(1 - t / 0.35, 0, 1) ** 2
    s += np.sin(2 * np.pi * 60 * t).astype(np.float32) * 0.15 * np.exp(-np.linspace(0, 14, n))
    sig = reverb(normalize(s, 0.6), decay=1.1, wet=0.3, seed=163)
    save("led_on", fade(normalize(sig, 0.55), 0.002, 0.1))


def power_down():
    dur = 3.2
    n = int(dur * SR)
    t = np.arange(n) / SR
    f = 220.0 * np.exp(-t * 1.1) + 22.0
    s = np.sin(2 * np.pi * np.cumsum(f) / SR).astype(np.float32)
    s += np.sin(2 * np.pi * np.cumsum(f * 2.01) / SR).astype(np.float32) * 0.3
    s *= np.clip(1.0 - t / dur, 0, 1) ** 1.4
    hiss = band(spectral_noise(n, alpha=1.0, seed=171), lo=400, hi=5000) * np.clip(
        1.0 - t / dur, 0, 1
    ) ** 2 * 0.15
    sig = normalize(s * 0.8 + hiss, 0.8)
    save("power_down", fade(normalize(sig, 0.8), 0.01, 0.4))


def ui_click():
    dur = 0.12
    n = int(dur * SR)
    s = np.sin(2 * np.pi * 700 * np.arange(n) / SR).astype(np.float32) * np.exp(-np.linspace(0, 40, n))
    s = band(s, lo=200, hi=6000)
    save("ui_click", fade(normalize(s, 0.3), 0.001, 0.03))


def ui_hover():
    dur = 0.16
    n = int(dur * SR)
    s = np.sin(2 * np.pi * 1100 * np.arange(n) / SR).astype(np.float32) * np.exp(-np.linspace(0, 30, n))
    s = band(s, lo=300, hi=8000)
    save("ui_hover", fade(normalize(s, 0.18), 0.002, 0.04))


def rain_stop():
    """The sound of rain being sucked out of the world - used in the finale."""
    dur = 4.0
    n = int(dur * SR)
    s = spectral_noise(n, alpha=0.8, seed=181)
    s = band(s, lo=250, hi=9000)
    env = np.clip(1.0 - np.arange(n) / n, 0, 1) ** 2.2
    s = s * env.astype(np.float32)
    save("rain_stop", fade(normalize(s, 0.5), 0.01, 0.2))


def metal_ring():
    dur = 1.4
    n = int(dur * SR)
    t = np.arange(n) / SR
    s = np.zeros(n, np.float32)
    for f, a, d in ((523, 0.5, 5.0), (781, 0.35, 6.5), (1207, 0.22, 8.0), (1693, 0.12, 9.0)):
        s += np.sin(2 * np.pi * f * t).astype(np.float32) * a * np.exp(-t * d)
    s = reverb(normalize(s, 0.7), decay=1.8, wet=0.4, seed=191)
    save("metal_ring", fade(normalize(s, 0.7), 0.001, 0.3))


def chair_scrape():
    dur = 1.1
    n = int(dur * SR)
    t = np.arange(n) / SR
    base = band(spectral_noise(n, alpha=1.4, seed=201), lo=200, hi=2200)
    mod = (0.4 + 0.6 * np.abs(np.sin(2 * np.pi * 8 * t))).astype(np.float32)
    env = np.hanning(n).astype(np.float32)
    s = base * mod * env
    s = resonance(s, 420.0, q=8.0)
    s = reverb(normalize(s, 0.7), decay=1.2, wet=0.3, seed=203)
    save("chair_scrape", fade(normalize(s, 0.7), 0.01, 0.1))


def main():
    print("synthesising audio into", OUT)
    rain_loop()
    wind_loop()
    tone_loop()
    gen_hum_loop()
    elev_loop()
    buzz_loop()
    thunder_set()
    drip_set()
    creak_set()
    steps_set()
    door_slam()
    door_creak()
    clock_tick()
    gear_clunk()
    beeps()
    lever_clunk()
    spark()
    generator_start()
    piano_note()
    stinger()
    heartbeat()
    paper("paper_pickup", 0.5, 211)
    paper("paper_flip", 0.35, 213, bright=11000.0)
    whisper()
    distant_steps()
    switch_click()
    led_on()
    power_down()
    ui_click()
    ui_hover()
    rain_stop()
    metal_ring()
    chair_scrape()
    total = sum(
        os.path.getsize(os.path.join(OUT, f)) for f in os.listdir(OUT) if f.endswith(".wav")
    )
    print(f"total audio: {total/1024/1024:.1f} MB")


if __name__ == "__main__":
    main()
