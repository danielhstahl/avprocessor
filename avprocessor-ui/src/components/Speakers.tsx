import { Space, Typography, InputNumber, Row, Col, Switch } from 'antd';
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import { Speaker } from '../state/speaker'
import { DelayType } from '../state/delay';
const { Text } = Typography

const inputStyle = { width: "100%" }
type PartialProps = {
    speaker: Speaker,
    updateSpeaker: (speaker: Speaker) => void
}
const CrossoverAction = ({ speaker, updateSpeaker }: PartialProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text ellipsis={true}>Crossover:</Text>
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
    const title = delayType === DelayType.MS ? "Delay:" : "Distance:"
    return <Space direction="horizontal" size="middle" >
        <Text ellipsis={true}>{title}</Text>
        <InputNumber
            style={inputStyle}
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
        <Text ellipsis={true}>Trim:</Text>
        <InputNumber
            style={inputStyle}
            value={speaker.gain}
            onChange={v => v !== null && updateSpeaker({ ...speaker, gain: v })}
            min={-10}
            max={10}
            {...floatFormatter("db")}
        />
    </Space>
}

const SpeakerRecord = ({ speaker, updateSpeaker, delayType }: SpeakerProps) => {
    return <Row gutter={[12, 12]} style={{ minHeight: 100 }} justify="space-evenly">
        <Col xs={24} lg={8}>{!speaker.isSubwoofer ? <CrossoverAction speaker={speaker} updateSpeaker={updateSpeaker} /> : <div></div>}</Col>
        <Col xs={24} lg={8}><DelayAction speaker={speaker} updateSpeaker={updateSpeaker} delayType={delayType} /></Col>
        <Col xs={24} lg={8}><TrimAction speaker={speaker} updateSpeaker={updateSpeaker} /></Col>
    </Row >
}

export default SpeakerRecord

