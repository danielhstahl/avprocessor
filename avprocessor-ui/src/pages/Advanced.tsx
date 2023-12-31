import { List, message, Typography, Row, Col, Select } from 'antd';
import { DeleteOutlined } from '@ant-design/icons'
import React from 'react';
import { Version, useVersion, VersionAction } from '../state/version'
import { deleteConfig } from '../services/configuration';
import { DelayAction, useDelay, DelayType } from '../state/delay';
import { DeviceAction, useDevice, DeviceType } from '../state/device';
const { Text, Title } = Typography

const DEVICE_LABELS = [
    {
        value: DeviceType.MotuMk5,
        label: "Motu UltraLite MK5"
    },
    {
        value: DeviceType.OktoDac8,
        label: "Octo DAC8 Pro"
    },
    {
        value: DeviceType.ToppingDm7,
        label: "Topping DM7"
    },
    {
        value: DeviceType.HDMI,
        label: "Generic HDMI"
    }
]

const AdvancedComponent: React.FC = () => {
    const { state: { versions }, dispatch: versionDispatch } = useVersion()
    const { state: { delayType }, dispatch: delayTypeDispatch } = useDelay()
    const { state: { deviceType }, dispatch: deviceTypeDispatch } = useDevice()
    const [messageApi, contextHolder] = message.useMessage()

    const deleteSuccess = () => {
        messageApi.success("Configuration Deleted")
    }
    const deleteFailure = () => {
        messageApi.error("Something went wrong!")
    }

    const onRemove = (version: number) => deleteConfig(version)
        .then(() => versionDispatch({ type: VersionAction.REMOVE, value: version }))
        .then(deleteSuccess)
        .catch(deleteFailure)
    return <>
        {contextHolder}
        <Row>
            <Col xs={24}>
                <Title level={4}>Select Device</Title>
                <Select
                    value={deviceType}
                    onChange={v => deviceTypeDispatch({ type: DeviceAction.UPDATE, value: v })}
                    options={DEVICE_LABELS}
                    style={{ width: '100%' }} />
            </Col>
            <Col xs={24} md={12}>
                <Title level={4}>Configuration Versions</Title>
                <List
                    itemLayout="horizontal"
                    dataSource={versions}
                    renderItem={(version: Version) => <List.Item
                        actions={[<DeleteOutlined onClick={() => onRemove(version.version)} />]}
                    >
                        <Text strong={version.appliedVersion}>{`Version: ${version.version} (${version.versionDate}) ${version.appliedVersion ? "(Currently applied)" : ""}`}</Text>

                    </List.Item>}
                />
            </Col>
            <Col xs={24} md={12}>
                <Title level={4}>Distance/Delay</Title>
                <Select
                    value={delayType}
                    onChange={v => delayTypeDispatch({ type: DelayAction.UPDATE, value: v })}
                    options={Object.values(DelayType).map(v => ({ value: v, label: v }))}
                    style={{ width: '100%' }} />
            </Col>
        </Row>
    </>
}

export default AdvancedComponent;