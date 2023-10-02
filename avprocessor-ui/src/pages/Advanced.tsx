import { List, message, Typography, Row, Col, Select } from 'antd';
import { DeleteOutlined } from '@ant-design/icons'
import React from 'react';
import { Version, useVersion, VersionAction } from '../state/version'
import { deleteConfig } from '../services/configuration';
import { DelayAction, useDelay, DelayType } from '../state/delay';
const { Text, Title } = Typography

//add clear database
const AdvancedComponent: React.FC = () => {
    const { state: { versions }, dispatch: versionDispatch } = useVersion()
    const { state: { delayType }, dispatch: delayTypeDispatch } = useDelay()
    const [messageApi, contextHolder] = message.useMessage()

    const saveSuccess = () => {
        messageApi.success("Configuration Deleted")
    }
    const saveFailure = () => {
        messageApi.error("Something went wrong!")
    }

    const onRemove = (version: number) => deleteConfig(version)
        .then(() => versionDispatch({ type: VersionAction.REMOVE, value: version }))
        .then(saveSuccess)
        .catch(saveFailure)
    return <>
        {contextHolder}
        <Row>
            <Col xs={24} md={12} lg={8}>
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
            <Col xs={24} md={12} lg={8}>
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