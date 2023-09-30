import { List, Space, message, Typography } from 'antd';
import { DeleteOutlined } from '@ant-design/icons'
import React from 'react';
import { Version, useVersion, VersionAction } from '../state/version'
import { deleteConfig } from '../services/configuration';
const { Text } = Typography

//add ms/ft/meters selection
//add clear database
const AdvancedComponent: React.FC = () => {
    const { state: { versions }, dispatch: versionDispatch } = useVersion()

    const [messageApi, contextHolder] = message.useMessage()

    const saveSuccess = () => {
        messageApi.success("Configuration Deleted")
    }
    const saveFailure = () => {
        messageApi.error("Something went wrong!")
    }

    const onRemove = (version: string) => deleteConfig(version)
        .then((value) => versionDispatch({ type: VersionAction.REMOVE, value }))
        .then(saveSuccess)
        .catch(saveFailure)
    return <Space direction="vertical" size="middle" style={{ display: 'flex' }}>
        {contextHolder}
        <List
            itemLayout="horizontal"
            dataSource={versions}
            renderItem={(version: Version) => <List.Item
                actions={[<DeleteOutlined onClick={() => onRemove(version.version)} />]}
            >
                <Text strong={version.appliedVersion}>{`Version: ${version.version} ${version.appliedVersion ? "(Currently applied)" : ""}`}</Text>

            </List.Item>}
        />
    </Space>
}

export default AdvancedComponent;