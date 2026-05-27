export function playMoveSound(soundMuted: boolean) {
  if (soundMuted) return
  const AudioContextConstructor = window.AudioContext
  if (!AudioContextConstructor) return

  const context = new AudioContextConstructor()
  const now = context.currentTime
  const output = context.createGain()
  output.gain.setValueAtTime(0.0001, now)
  output.gain.exponentialRampToValueAtTime(0.18, now + 0.006)
  output.gain.exponentialRampToValueAtTime(0.0001, now + 0.13)
  output.connect(context.destination)

  for (const [index, frequency] of [720, 410].entries()) {
    const oscillator = context.createOscillator()
    oscillator.type = 'triangle'
    oscillator.frequency.setValueAtTime(frequency, now + index * 0.035)
    oscillator.connect(output)
    oscillator.start(now + index * 0.035)
    oscillator.stop(now + 0.115 + index * 0.035)
  }

  window.setTimeout(() => {
    void context.close()
  }, 220)
}
