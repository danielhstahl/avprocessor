import { Button, Slider, Progress, Row, Col, Space } from 'antd';
import { MinusOutlined, PlusOutlined } from '@ant-design/icons';
import { useEffect, useState } from 'react';
import useWebSocket from 'react-use-websocket';

interface VolumeInputs {
    wsPort: number
}
type VolumeGet = {
    GetVolume: {
        value: number
    }
}
const MIN_VOLUME = -100
const MAX_VOLUME = 0
const VOLUME_STEP = 1
const centerStyle = { width: '100%', justifyContent: 'center' }
const convertTo100 = (volume: number) => 100 * ((volume - MIN_VOLUME) / (MAX_VOLUME - MIN_VOLUME))
const VolumeCard = ({ wsPort }: VolumeInputs) => {
    const socketUrl = `ws://${window.location.host.split(":")[0]}:${wsPort}`
    const [volume, setVolume] = useState(0);

    const { sendJsonMessage, lastJsonMessage } = useWebSocket(socketUrl);

    useEffect(() => {
        if (lastJsonMessage) {
            if (lastJsonMessage.hasOwnProperty("GetVolume")) {
                setVolume((lastJsonMessage as VolumeGet).GetVolume.value)
            }
        }
    }, [lastJsonMessage]);
    useEffect(() => {
        setInterval(() => {
            sendJsonMessage("GetVolume");
        }, 3000)
    }, [sendJsonMessage])

    const onVolumeChange = (vol: number) => {
        setVolume(vol) //optimistic
        sendJsonMessage({ SetVolume: vol });
    }

    return <Row align="middle" justify="center">
        <Col xs={24}>
            <Row justify="space-evenly">
                <Col xs={2}>
                    <Space direction="horizontal" style={centerStyle}>
                        <Button onClick={() => onVolumeChange(volume - VOLUME_STEP)} shape="circle" icon={<MinusOutlined />} />
                    </Space>
                </Col>
                <Col xs={20}>
                    <Slider value={volume} min={MIN_VOLUME} max={MAX_VOLUME} step={VOLUME_STEP} onChange={onVolumeChange} />
                </Col>
                <Col xs={2}>
                    <Space direction="horizontal" style={centerStyle}>
                        <Button onClick={() => onVolumeChange(volume + VOLUME_STEP)} shape="circle" icon={<PlusOutlined />} />
                    </Space>
                </Col>
            </Row>
        </Col>
        <Col xs={24}>
            <Space direction="horizontal" style={centerStyle}>
                <Progress type="circle" format={() => volume} percent={convertTo100(volume)} />
            </Space>
        </Col>
    </Row>
}
export default VolumeCard