import { List, Select, Space, Typography, Card, Button, message, Row, Col } from 'antd';
import { Speaker, SPEAKER_OPTIONS } from '../state/speaker'
import React, { useContext, useState } from 'react';
import { SpeakerContext } from '../state/speaker'
import { FilterContext, FilterWithIndex } from '../state/filter'
import SpeakerRecord, { SpeakerProps } from '../components/Speakers'
import PeqRecord, { PeqProps } from '../components/Peq'
import { VersionContext } from '../state/version'
import { applyConfig, saveConfig, getConfiguration, ConfigPayload } from '../services/configuration';

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
interface SpeakerComponentProps {
    getConfigurationProp?: (_: string) => Promise<ConfigPayload>
}
const SpeakerComponent: React.FC<SpeakerComponentProps> = ({ getConfigurationProp = getConfiguration }: SpeakerComponentProps) => {
    const { speakers, speakerConfiguration, setSpeakerConfiguration, setSpeakerBase, updateSpeaker, setSpeakers, } = useContext(SpeakerContext)
    const { addVersion, setSelectedVersion, selectedVersion, setAppliedVersion, versions } = useContext(VersionContext)
    const { setFilterBase } = useContext(FilterContext)
    const { filters, updateFilter, addFilter, removeFilter, setFilters } = useContext(FilterContext)
    const speakerFilters = perSpeakerFilters(filters)

    const [messageApi, contextHolder] = message.useMessage()

    const saveSuccess = () => {
        messageApi.success("Configuration Saved")
    }
    const saveFailure = () => {
        messageApi.error("Something went wrong!")
    }

    const applySuccess = () => {
        messageApi.success("Configuration Applied")
    }

    const onApply = () => {
        if (selectedVersion) {
            applyConfig(selectedVersion).then(setAppliedVersion).then(applySuccess).catch(saveFailure)
        }
    }
    const onSave = () => saveConfig({ speakers, filters })
        .then(v => {
            addVersion(v)
            setSelectedVersion(v)
        })
        .then(saveSuccess)
        .catch(saveFailure)

    const onSelectVersion = (version: string) => {
        setSelectedVersion(version)
        getConfigurationProp(version).then(({ filters, speakers }) => {
            if (speakers && speakers.length > 0) {
                setSpeakers(speakers) //this will trigger a `setSpeakerBase` and `setFilterBase` since it will update the speakerConfiguration
                setFilters(filters)
            }
        })
    }
    return <>
        <Row style={{ paddingTop: 20 }}>
            <Col xs={6}>
                <Text strong>Select Configuration Version</Text>
            </Col>
            <Col xs={18}>
                <Select value={selectedVersion} onChange={onSelectVersion} options={versions.map(({ version }) => ({ value: version, label: version }))} style={{ width: '100%' }} />
            </Col>
        </Row>
        <Row style={{ paddingTop: 20, paddingBottom: 20 }}>
            <Col xs={6}>
                <Text strong>Select Speaker Layout</Text>
            </Col>
            <Col xs={18}>
                <Select
                    value={speakerConfiguration}
                    onChange={v => {
                        setSpeakerConfiguration(v)
                        setSpeakerBase(v)
                        setFilterBase(v)
                    }}
                    options={SPEAKER_OPTIONS.map(({ label }) => ({ value: label, label }))}
                    style={{ width: '100%' }}
                />
            </Col>
        </Row>

        {contextHolder}

        <List
            itemLayout="vertical"
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
        <Space direction="horizontal" size="middle" style={{ display: 'flex', paddingTop: 20 }}>
            <Button type="primary" onClick={onSave}>Save</Button>
            {selectedVersion && <Button type="primary" onClick={onApply}>Apply Configuration</Button>}
        </Space>
    </>
}

export default SpeakerComponent;