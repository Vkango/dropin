// Curated, built-in summaries from the BASS Help material.
// Keep the explanatory text in i18n; this module only stores stable effect IDs
// and technical range/default metadata so the app has no runtime document dependency.
const entries = {
  'dx8.compressor': ['dx8Compressor', { fGain: ['-60…60 dB', '0 dB'], fAttack: ['0.01…500 ms', '10 ms'], fRelease: ['50…3000 ms', '200 ms'], fThreshold: ['-60…0 dB', '-20 dB'], fRatio: ['1…100', '3'], fPredelay: ['0…4 ms', '4 ms'] }],
  'bassFx.compressor2': ['bassFxCompressor2', { fGain: ['-60…60 dB', '0 dB'], fThreshold: ['-60…0 dB', '-15 dB'], fRatio: ['1…n', '3'], fAttack: ['0.01…1000 ms', '10 ms'], fRelease: ['0.01…5000 ms', '200 ms'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.damp': ['bassFxDamp', { fTarget: ['0…1', '—'], fQuiet: ['0…1', '—'], fRate: ['0…1', '—'], fGain: ['0…n', '—'], fDelay: ['0…n seconds', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  volume: ['volume', { fTarget: ['0…>1', '1'], fCurrent: ['-1…>1', '1'], fTime: ['0…n seconds', '0'], lCurve: ['0 or 1', '0'] }],
  'bassFx.volume': ['bassFxVolume', { lChannel: ['BASS_FX channel flags', '—'], fVolume: ['0…n', '1'] }],
  'bassFx.volumeenvelope': ['volumeEnvelope', { lChannel: ['BASS_FX channel flags', '—'], lNodeCount: ['at least 1', '—'], pNodes: ['position + level nodes', '—'], bFollow: ['TRUE / FALSE', 'FALSE'] }],
  'dx8.reverb': ['dx8Reverb', { fInGain: ['-96…0 dB', '0 dB'], fReverbMix: ['-96…0 dB', '0 dB'], fReverbTime: ['0.001…3000 ms', '1000 ms'], fHighFreqRTRatio: ['0.001…0.999', '0.001'] }],
  'dx8.i3dl2reverb': ['dx8I3dl2Reverb', { lRoom: ['-10000…0 mB', '-1000'], lRoomHF: ['-10000…0 mB', '-100'], flRoomRolloffFactor: ['0…10', '0'], flDecayTime: ['0.1…20 s', '1.49 s'], flDecayHFRatio: ['0.1…2', '0.83'], flReflectionsDelay: ['0…0.3 s', '0.007 s'], flReverbDelay: ['0…0.1 s', '0.011 s'], flDiffusion: ['0…100%', '100%'], flDensity: ['0…100%', '100%'], flHFReference: ['20…20000 Hz', '5000 Hz'] }],
  'bassFx.freeverb': ['freeverb', { fDryMix: ['0…1', '0'], fWetMix: ['0…3', '1'], fRoomSize: ['0…1', '0.5'], fDamp: ['0…1', '0.5'], fWidth: ['0…1', '1'], lMode: ['0 or FREEZE', '0'], lChannel: ['BASS_FX channel flags', '—'] }],
  'dx8.echo': ['dx8Echo', { fWetDryMix: ['0…100', '50'], fFeedback: ['0…100', '50'], fLeftDelay: ['1…2000 ms', '500 ms'], fRightDelay: ['1…2000 ms', '500 ms'], lPanDelay: ['TRUE / FALSE', 'FALSE'] }],
  'bassFx.echo': ['bassFxEcho', { fLevel: ['linear level', '—'], lDelay: ['milliseconds', '—'] }],
  'bassFx.echo2': ['bassFxEcho2', { fDryMix: ['-2…2', '—'], fWetMix: ['-2…2', '—'], fFeedback: ['-1…1', '—'], fDelay: ['seconds', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.echo3': ['bassFxEcho3', { fDryMix: ['-2…2', '—'], fWetMix: ['-2…2', '—'], fDelay: ['seconds', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.echo4': ['bassFxEcho4', { fDryMix: ['-2…2', '—'], fWetMix: ['-2…2', '—'], fFeedback: ['-1…1', '—'], fDelay: ['0…n seconds', '—'], bStereo: ['TRUE / FALSE', 'FALSE'], lChannel: ['BASS_FX channel flags', '—'] }],
  'dx8.chorus': ['dx8Chorus', { fWetDryMix: ['0…100', '50'], fDepth: ['0…100', '10'], fFeedback: ['-99…99', '25'], fFrequency: ['0…10 Hz', '1.1'], lWaveform: ['0 or 1', '1'], fDelay: ['0…20 ms', '16 ms'] }],
  'dx8.flanger': ['dx8Flanger', { fWetDryMix: ['0…100', '50'], fDepth: ['0…100', '100'], fFeedback: ['-99…99', '-50'], fFrequency: ['0…10 Hz', '0.25'], fDelay: ['0…4 ms', '2 ms'] }],
  'bassFx.flanger': ['bassFxFlanger', { fWetDry: ['-2…2', '—'], fSpeed: ['modulation speed', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.chorus': ['bassFxChorus', { fDryMix: ['-2…2', '—'], fWetMix: ['-2…2', '—'], fFeedback: ['-1…1', '—'], fMinSweep: ['0…6000 ms', '—'], fMaxSweep: ['0…6000 ms', '—'], fRate: ['0…1000 ms/s', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.phaser': ['phaser', { fDryMix: ['-2…2', '—'], fWetMix: ['-2…2', '—'], fFeedback: ['-1…1', '—'], fRate: ['0…10 Hz', '—'], fRange: ['0…10 octaves', '—'], fFreq: ['0…1000 Hz', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.autowah': ['autowah', { fRate: ['0…10 Hz', '—'], fRange: ['0…10 octaves', '—'], fFreq: ['0…1000 Hz', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'dx8.gargle': ['gargle', { dwRateHz: ['1…1000 Hz', '20'], dwWaveShape: ['0 or 1', '0'] }],
  'bassFx.rotate': ['rotate', { fRate: ['Hz; sign sets direction', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.reverb': ['bassFxReverb', { fLevel: ['linear level', '—'], lDelay: ['milliseconds', '—'] }],
  'bassFx.lowpassfilter': ['lowpassFilter', { fResonance: ['0…1', '—'], fCutOffFreq: ['Hz', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.allpassfilter': ['allpassFilter', { fGain: ['linear gain', '—'], fDelay: ['seconds', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.compressor': ['bassFxCompressor', { fThreshold: ['dB', '—'], fAttacktime: ['milliseconds', '—'], fReleasetime: ['milliseconds', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'dx8.distortion': ['dx8Distortion', { fGain: ['-60…0 dB', '-18 dB'], fEdge: ['0…100%', '15%'], fPostEQCenterFrequency: ['100…8000 Hz', '2400 Hz'], fPostEQBandwidth: ['100…8000 Hz', '2400 Hz'], fPreLowpassCutoff: ['100…8000 Hz', '8000 Hz'] }],
  'bassFx.distortion': ['bassFxDistortion', { fDrive: ['0…5', '—'], fDryMix: ['-5…5', '—'], fWetMix: ['-5…5', '—'], fFeedback: ['-1…1', '—'], fVolume: ['0…2', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'dx8.parameq': ['dx8Parameq', { fCenter: ['80…16000 Hz on Windows', '—'], fBandwidth: ['1…36 semitones', '12'], fGain: ['-15…15 dB', '0'] }],
  'bassFx.peakeq': ['bassFxPeakeq', { lBand: ['0…n', '—'], fBandwidth: ['0.1…10 octaves', '—'], fQ: ['0…1', '—'], fCenter: ['1 Hz…sample rate/2', '—'], fGain: ['-15…15 dB', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.biquadfilter': ['biquadFilter', { lFilter: ['filter enum', '—'], fCenter: ['1 Hz…sample rate/2', '—'], fGain: ['-15…15 dB', '—'], fBandwidth: ['0.1…10', '—'], fQ: ['0.1…1', '—'], fS: ['0.1…1', '—'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.pitchshift': ['pitchshift', { fPitchShift: ['0.5…2', '1'], fSemitones: ['semitones', '0'], lFFTsize: ['1024…8192; power of 2', '2048'], lOsamp: ['4…32', '8'], lChannel: ['BASS_FX channel flags', '—'] }],
  'bassFx.mix': ['mix', { lChannel: ['integer channel map', '—'] }]
}

const glossary = [
  ['dry', 'Dry'], ['wet', 'Wet'], ['wetDry', 'Wet/dry mix'], ['lfo', 'LFO'], ['feedback', 'Feedback'], ['gain', 'Gain'], ['db', 'dB'], ['hz', 'Hz'], ['ms', 'ms']
]

export const getEffectWiki = (kind) => {
  const entry = entries[kind]
  if (!entry) return null
  const [name, parameters] = entry
  return {
    purposeKey: `effects.wikiEntries.${name}.purpose`,
    quickStartKey: `effects.wikiEntries.${name}.quickStart`,
    parameters: Object.fromEntries(Object.entries(parameters).map(([key, [range, defaultValue]]) => [key, { range, defaultValue }]))
  }
}

export const getWikiGlossary = () => glossary.map(([name, label]) => ({
  titleKey: `effects.glossary.${name}.title`,
  descriptionKey: `effects.glossary.${name}.description`,
  fallbackTitle: label
}))
