import { List, Space, message, Typography } from 'antd';
import { DeleteOutlined } from '@ant-design/icons'
import React, { useContext, } from 'react';
import { Version, VersionContext } from '../state/version'
import { deleteConfig } from '../services/configuration';
const { Text } = Typography

const AdvancedComponent: React.FC = () => {
    const { versions, removeVersion } = useContext(VersionContext)

    const [messageApi, contextHolder] = message.useMessage()

    const saveSuccess = () => {
        messageApi.success("Configuration Deleted")
    }
    const saveFailure = () => {
        messageApi.error("Something went wrong!")
    }

    const onRemove = (version: string) => deleteConfig(version)
        .then(removeVersion)
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

/**{version.appliedVersion ?}
                <List.Item.Meta
                    title={`Version: ${version.version}`}
                    description={version.appliedVersion && "Currently applied"}
                /> */

export default AdvancedComponent;