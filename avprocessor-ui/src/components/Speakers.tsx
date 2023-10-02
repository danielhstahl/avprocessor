import { Space, Typography, InputNumber, Row, Col, Switch } from 'antd';
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import { Speaker } from '../state/speaker'
import { DelayType } from '../state/delay';
const { Text } = Typography


type PartialProps = {
    speaker: Speaker,
    updateSpeaker: (speaker: Speaker) => void
}
const CrossoverAction = ({ speaker, updateSpeaker }: PartialProps) => {
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

export interface SpeakerProps extends PartialProps {
    delayType: DelayType
}
const DelayAction = ({ speaker, updateSpeaker, delayType }: SpeakerProps) => {
    let title
    switch (delayType) {
        case DelayType.MS:
            title = "Delay:"
            break
        default:
            title = "Distance:"
    }
    return <Space direction="horizontal" size="middle" >
        <Text>{title}</Text>
        <InputNumber
            value={speaker.distance}
            onChange={v => v !== null && updateSpeaker({ ...speaker, distance: v })}
            min={0}
            max={1000}
            step="0.5"
            {...floatFormatter(delayType)}
        />
    </Space>
}

const TrimAction = ({ speaker, updateSpeaker }: PartialProps) => {
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

const SpeakerRecord = ({ speaker, updateSpeaker, delayType }: SpeakerProps) => {
    return <Row style={{ minHeight: 100 }} justify="space-evenly">
        <Col xs={8}>{!speaker.isSubwoofer ? <CrossoverAction speaker={speaker} updateSpeaker={updateSpeaker} /> : <div></div>}</Col>
        <Col xs={8}><DelayAction speaker={speaker} updateSpeaker={updateSpeaker} delayType={delayType} /></Col>
        <Col xs={8}><TrimAction speaker={speaker} updateSpeaker={updateSpeaker} /></Col>
    </Row >
}

export default SpeakerRecord

