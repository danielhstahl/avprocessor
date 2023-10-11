import { Typography, InputNumber, Row, Col, Switch } from 'antd';
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import { Speaker } from '../state/speaker'
import { DelayType } from '../state/delay'
import { inputStyle, textStyle } from "./styles"
const { Text } = Typography

type PartialProps = {
    speaker: Speaker,
    updateSpeaker: (speaker: Speaker) => void
}
const CrossoverAction = ({ speaker, updateSpeaker }: PartialProps) => {
    return <Row align="middle" justify="center">
        <Col xs={0} md={9}>
            <Text style={textStyle} ellipsis={true}>Crossover:</Text>
        </Col>
        <Col xs={9} md={0}>
            <Text style={textStyle} ellipsis={true}>Xover:</Text>
        </Col>
        <Col xs={5}>
            <Switch checked={speaker.crossover !== null} onChange={v => updateSpeaker({ ...speaker, crossover: v ? 0 : null })} />
        </Col>
        <Col xs={10}>
            <InputNumber
                style={inputStyle}
                disabled={speaker.crossover === null}
                value={speaker.crossover}
                onChange={v => updateSpeaker({ ...speaker, crossover: v })}
                min={0}
                max={1000}
                {...intFormatter("hz")}
            />
        </Col>
    </Row>
}
export interface SpeakerProps extends PartialProps {
    delayType: DelayType
}
const DelayAction = ({ speaker, updateSpeaker, delayType }: SpeakerProps) => {
    const title = delayType === DelayType.MS ? "Delay:" : "Distance:"
    return <Row align="middle" justify="center">
        <Col xs={9}>
            <Text style={{ float: "right", paddingRight: 12 }} ellipsis={true}>{title}</Text>
        </Col>
        <Col xs={15}>
            <InputNumber
                style={inputStyle}
                value={speaker.distance}
                onChange={v => v !== null && updateSpeaker({ ...speaker, distance: v })}
                min={0}
                max={1000}
                step="0.5"
                {...floatFormatter(delayType)}
            />
        </Col>
    </Row>
}

const TrimAction = ({ speaker, updateSpeaker }: PartialProps) => {
    return <Row align="middle" justify="center">
        <Col xs={9}>
            <Text style={textStyle} ellipsis={true}>Trim:</Text>
        </Col>
        <Col xs={15}>
            <InputNumber
                style={inputStyle}
                value={speaker.gain}
                onChange={v => v !== null && updateSpeaker({ ...speaker, gain: v })}
                min={-10}
                max={10}
                {...floatFormatter("db")}
            />
        </Col>
    </Row>
}

const SpeakerRecord = ({ speaker, updateSpeaker, delayType }: SpeakerProps) => {
    return <Row gutter={[12, 12]} style={{ minHeight: 100 }} justify="space-evenly">
        <Col xs={24} lg={8}>{!speaker.isSubwoofer ? <CrossoverAction speaker={speaker} updateSpeaker={updateSpeaker} /> : <div></div>}</Col>
        <Col xs={24} lg={8}><DelayAction speaker={speaker} updateSpeaker={updateSpeaker} delayType={delayType} /></Col>
        <Col xs={24} lg={8}><TrimAction speaker={speaker} updateSpeaker={updateSpeaker} /></Col>
    </Row >
}

export default SpeakerRecord

