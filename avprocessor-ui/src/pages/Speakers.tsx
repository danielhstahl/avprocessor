import { List, Select, Space, Typography, InputNumber } from 'antd';
import { Speaker, SPEAKER_OPTIONS } from '../state/speaker'
import { Switch } from 'antd';
import React, { useContext } from 'react';
import { SpeakerContext } from '../state/speaker'
import { FilterContext } from '../state/filter'
import { floatFormatter, intFormatter } from '../utils/inputParsers';

const { Text } = Typography


type ActionProps = {
    speaker: Speaker,
    updateSpeaker: (speaker: Speaker) => void
}
const CrossoverAction = ({ speaker, updateSpeaker }: ActionProps) => {
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

const DelayAction = ({ speaker, updateSpeaker }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Delay:</Text>
        <InputNumber
            value={speaker.delay}
            onChange={v => v && updateSpeaker({ ...speaker, delay: v })}
            min={0}
            max={1000}
            step="0.5"
            {...floatFormatter("ms")}
        />
    </Space>
}

const TrimAction = ({ speaker, updateSpeaker }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Trim:</Text>
        <InputNumber
            value={speaker.gain}
            onChange={v => v && updateSpeaker({ ...speaker, gain: v })}
            min={-10}
            max={10}
            {...floatFormatter("db")}
        />
    </Space>
}

const SpeakerComponent: React.FC = () => {
    const { speakers, speakerConfiguration, setSpeakerBase, updateSpeaker } = useContext(SpeakerContext)
    const { setFilterBase } = useContext(FilterContext)
    return <Space direction="vertical" size="middle" style={{ display: 'flex' }}>
        <Space direction="horizontal" size="middle" style={{ display: 'flex' }}>
            <Text strong>Select Speaker Layout</Text>
            <Select
                value={speakerConfiguration}
                onChange={v => {
                    setSpeakerBase(v)
                    setFilterBase(v)
                }}
                options={SPEAKER_OPTIONS.map(({ label }) => ({ value: label, label }))}
                style={{ width: '100%' }}
            />
        </Space>

        <List
            itemLayout="horizontal"
            dataSource={speakers}
            renderItem={(speaker: Speaker) => (
                <List.Item
                    actions={[
                        !speaker.isSubwoofer && <CrossoverAction speaker={speaker} updateSpeaker={updateSpeaker} />,
                        <DelayAction speaker={speaker} updateSpeaker={updateSpeaker} />,
                        <TrimAction speaker={speaker} updateSpeaker={updateSpeaker} />
                    ]}
                >
                    <List.Item.Meta
                        //todo, put picture of speaker type here.  need to "add speaker type" entry
                        //avatar={<Avatar src={`https://xsgames.co/randomusers/avatar.php?g=pixel&key=0`} />}
                        title={speaker.speaker}
                    //description="Left Speaker"
                    />
                </List.Item>
            )}
        />
    </Space>
}

export default SpeakerComponent;