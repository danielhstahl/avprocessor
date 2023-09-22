import { List, Select, Space, Typography, Card, Button } from 'antd';
import { Speaker, SPEAKER_OPTIONS } from '../state/speaker'
import React, { useContext, useState } from 'react';
import { SpeakerContext } from '../state/speaker'
import { FilterContext, FilterWithIndex, Filter } from '../state/filter'
import SpeakerRecord, { SpeakerProps } from '../components/Speakers'
import PeqRecord, { PeqProps } from '../components/Peq'
import { VersionContext } from '../state/version'
const { Text } = Typography

const tabList = [
    {
        label: "Speaker",
        key: "speaker"
    },
    {
        label: "PEQ",
        key: "peq"
    }
]

type ConfigPayload = { speakers: Speaker[], filters: Filter[] }

const saveConfig = (body: ConfigPayload, setVersion: (_: string) => void) => {
    fetch(`/config`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body)
    }).then(r => r.text()).then(setVersion)
}

const SpeakerCard = ({
    speaker,
    filters,
    updateFilter,
    removeFilter,
    updateSpeaker,
    addFilter
}: SpeakerProps & PeqProps) => {
    const [activeKey, setActiveKey] = useState(tabList[0].key)
    return <Card
        style={{ width: '100%' }}
        title={speaker.speaker}
        tabList={tabList}
        activeTabKey={activeKey}
        onTabChange={setActiveKey}
    >
        {activeKey === "speaker" ?
            <SpeakerRecord speaker={speaker} updateSpeaker={updateSpeaker} /> :
            <PeqRecord
                filters={filters}
                updateFilter={updateFilter}
                removeFilter={removeFilter}
                addFilter={addFilter}
            />
        }
    </Card>
}

type SpeakerFilter = Record<string, FilterWithIndex[]>
const perSpeakerFilters: (filters: FilterWithIndex[]) => SpeakerFilter = (filters: FilterWithIndex[]) => {
    return filters.reduce<SpeakerFilter>((agg, filter) => {
        return {
            ...agg,
            [filter.speaker]: agg[filter.speaker] ? [...agg[filter.speaker], filter] : [filter]
        }
    }, {})
}

// TODO add "save" functionality
const SpeakerComponent: React.FC = () => {
    const { speakers, speakerConfiguration, setSpeakerBase, updateSpeaker } = useContext(SpeakerContext)
    const { addVersion, setSelectedVersion } = useContext(VersionContext)
    const { setFilterBase } = useContext(FilterContext)
    const { filters, updateFilter, addFilter, removeFilter } = useContext(FilterContext)
    const speakerFilters = perSpeakerFilters(filters)
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
            renderItem={(speaker: Speaker) => <SpeakerCard
                speaker={speaker}
                updateSpeaker={updateSpeaker}
                filters={speakerFilters[speaker.speaker]}
                updateFilter={updateFilter}
                addFilter={() => addFilter(speaker.speaker)}
                removeFilter={removeFilter}
            />}
        />
        <Button onClick={() => saveConfig({ speakers, filters }, v => {
            addVersion(v)
            setSelectedVersion(v)
        })}>Save</Button>
    </Space>
}

export default SpeakerComponent;