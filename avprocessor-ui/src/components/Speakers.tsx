import { Space, Typography, InputNumber, Row, Col, Switch } from 'antd';
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import { Speaker } from '../state/speaker'
const { Text } = Typography


export type SpeakerProps = {
    speaker: Speaker,
    updateSpeaker: (speaker: Speaker) => void
}
const CrossoverAction = ({ speaker, updateSpeaker }: SpeakerProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Crossover:</Text>
        <Switch checked={speaker.crossover !== null} onChange={v => updateSpeaker({ ...speaker, crossover: v ? 0 : null })} />
        <InputNumber
            disabled={speaker.crossover === null}
            value={speaker.crossover}
            onChange={v => updateSpeaker({ ...speaker, crossover: v })}
            min={0}
            max={1000}
            {...intFormatter("hz")}
        />
    </Space>
}

const DelayAction = ({ speaker, updateSpeaker }: SpeakerProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Delay:</Text>
        <InputNumber
            value={speaker.delay}
            onChange={v => v !== null && updateSpeaker({ ...speaker, delay: v })}
            min={0}
            max={1000}
            step="0.5"
            {...floatFormatter("ms")}
        />
    </Space>
}

const TrimAction = ({ speaker, updateSpeaker }: SpeakerProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Trim:</Text>
        <InputNumber
            value={speaker.gain}
            onChange={v => v !== null && updateSpeaker({ ...speaker, gain: v })}
            min={-10}
            max={10}
            {...floatFormatter("db")}
        />
    </Space>
}

const SpeakerRecord = ({ speaker, updateSpeaker }: SpeakerProps) => {
    return <Row style={{ minHeight: 100 }} justify="space-evenly">
        <Col xs={8}>{!speaker.isSubwoofer ? <CrossoverAction speaker={speaker} updateSpeaker={updateSpeaker} /> : <div></div>}</Col>
        <Col xs={8}><DelayAction speaker={speaker} updateSpeaker={updateSpeaker} /></Col>
        <Col xs={8}><TrimAction speaker={speaker} updateSpeaker={updateSpeaker} /></Col>
    </Row >
}

export default SpeakerRecord

