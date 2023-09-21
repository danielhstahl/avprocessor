import { Filter } from '../state/filter'


const SAMPLE_RATE = 48000
const NUM_FREQUENCIES_TO_PLOT = 100
const constructFrequencies = () => {
    let frequencies = new Float32Array(NUM_FREQUENCIES_TO_PLOT);
    let nyquist = SAMPLE_RATE / 2;
    let minLog = 1;
    let maxLog = Math.log10(nyquist);

    return frequencies.map((_, i, a) => {
        let log = minLog + (i / a.length) * (maxLog - minLog);
        return Math.pow(10, log);
    })

}
export const constructVisualArray = (filters: Filter[]) => {
    const ac = new AudioContext({ sampleRate: SAMPLE_RATE })
    let magResponse = new Float32Array(NUM_FREQUENCIES_TO_PLOT);
    let phaseResponse = new Float32Array(NUM_FREQUENCIES_TO_PLOT);
    const freq = constructFrequencies()
    let freqResponse = new Array(NUM_FREQUENCIES_TO_PLOT).fill(0)
    filters.forEach(filter => {
        new BiquadFilterNode(ac, {
            type: "peaking",
            Q: filter.q,
            frequency: filter.freq,
            gain: filter.gain
        }).getFrequencyResponse(
            freq,
            magResponse,
            phaseResponse
        )
        magResponse.forEach((v, i) => {
            freqResponse[i] += Math.log10(v) //convert to DB
        })
    });
    freqResponse.forEach((v, i) => {
        freqResponse[i] = v * 20 //convert to DB
    })
    return { freq: Array.from(freq), freqResponse }
}
