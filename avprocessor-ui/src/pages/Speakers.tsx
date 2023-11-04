import { List, Select, Space, Typography, Card, Button, message, Row, Col } from 'antd';
import { Speaker, SPEAKER_OPTIONS } from '../state/speaker'
import React, { useState } from 'react';
import { useSpeaker, SpeakerAction } from '../state/speaker'
import { useFilter, FilterWithIndex, FilterAction } from '../state/filter'
import SpeakerRecord, { SpeakerProps } from '../components/Speakers'
import PeqRecord, { PeqProps } from '../components/Peq'
import { useVersion, VersionAction } from '../state/version'
import { applyConfig, saveConfig, getConfiguration, ConfigPayload } from '../services/configuration';
import { DelayAction, useDelay } from '../state/delay';
import { inputStyle } from '../components/styles'
import { DeviceAction, useDevice } from '../state/device';
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
    delayType,
    updateFilter,
    removeFilter,
    updateSpeaker,
    addFilter
}: SpeakerProps & PeqProps) => {
    const [activeKey, setActiveKey] = useState(tabList[0].key)
    return <Card
        style={inputStyle}
        title={speaker.speaker}
        tabList={tabList}
        activeTabKey={activeKey}
        onTabChange={setActiveKey}
    >
        {activeKey === "speaker" ?
            <SpeakerRecord speaker={speaker} updateSpeaker={updateSpeaker} delayType={delayType} /> :
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
    getConfigurationProp?: (_: number) => Promise<ConfigPayload>
}
const SpeakerComponent: React.FC<SpeakerComponentProps> = ({ getConfigurationProp = getConfiguration }: SpeakerComponentProps) => {
    const { state: { speakers, speakerConfiguration }, dispatch: speakerDispatch } = useSpeaker()
    const { state: { filters }, dispatch: filterDispatch } = useFilter()
    const { state: { versions, selectedVersion }, dispatch: versionDispatch } = useVersion()
    const { state: { delayType }, dispatch: delayDispatch } = useDelay()
    const { state: { deviceType }, dispatch: deviceDispatch } = useDevice()

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
            applyConfig(selectedVersion)
                .then(() => versionDispatch({ type: VersionAction.SET_APPLIED, value: selectedVersion }))
                .then(applySuccess).catch(saveFailure)
        }
    }
    const onSave = () => saveConfig({ speakers, filters, selectedDistance: delayType, device: deviceType })
        .then(value => {
            versionDispatch({ type: VersionAction.ADD, value })
            versionDispatch({ type: VersionAction.SELECT, value: value.version })
        })
        .then(saveSuccess)
        .catch(saveFailure)

    const onSelectVersion = (version: number) => {
        versionDispatch({ type: VersionAction.SELECT, value: version })
        getConfigurationProp(version).then(({ filters, speakers, selectedDistance, device }) => {
            if (speakers && speakers.length > 0) {
                speakerDispatch({ type: SpeakerAction.SET, value: speakers })
                filterDispatch({ type: FilterAction.SET, value: filters })
                delayDispatch({ type: DelayAction.UPDATE, value: selectedDistance })
                deviceDispatch({ type: DeviceAction.UPDATE, value: device })
            }
        })
    }
    return <>
        <Row style={{ paddingTop: 20 }} align="middle">
            <Col xs={6}>
                <Text strong>Select Configuration Version</Text>
            </Col>
            <Col xs={18}>
                <Select value={selectedVersion} onChange={onSelectVersion} options={versions.map(({ version, versionDate }) => ({
                    value: version, label: `${version} (${versionDate})`
                }))} style={inputStyle} />
            </Col>
        </Row>
        <Row style={{ paddingTop: 20, paddingBottom: 20 }} align="middle">
            <Col xs={6}>
                <Text strong>Select Speaker Layout</Text>
            </Col>
            <Col xs={18}>
                <Select
                    value={speakerConfiguration}
                    onChange={v => {
                        speakerDispatch({ type: SpeakerAction.CONFIG, value: v })
                        speakerDispatch({ type: SpeakerAction.INIT, value: v })
                        filterDispatch({ type: FilterAction.INIT, value: v })
                    }}
                    options={SPEAKER_OPTIONS.map(({ label }) => ({ value: label, label }))}
                    style={inputStyle}
                />
            </Col>
        </Row>
        {contextHolder}
        <List
            itemLayout="vertical"
            dataSource={speakers}
            renderItem={(speaker: Speaker) => <SpeakerCard
                delayType={delayType}
                speaker={speaker}
                updateSpeaker={(speaker: Speaker) => speakerDispatch({ type: SpeakerAction.UPDATE, value: speaker })}
                filters={speakerFilters[speaker.speaker]}
                updateFilter={(filter: FilterWithIndex) => filterDispatch({ type: FilterAction.UPDATE, value: filter })}
                addFilter={() => filterDispatch({ type: FilterAction.ADD, value: speaker.speaker })}
                removeFilter={(filter: FilterWithIndex) => filterDispatch({ type: FilterAction.REMOVE, value: filter })}
            />}
        />
        <Space direction="horizontal" size="middle" style={{ display: 'flex', paddingTop: 20 }}>
            <Button type="primary" onClick={onSave}>Save</Button>
            {selectedVersion && <Button type="primary" onClick={onApply}>Apply Configuration</Button>}
        </Space>
    </>
}

export default SpeakerComponent;