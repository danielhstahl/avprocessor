import VolumeCard from "../components/Volume"
import { Card, Space } from 'antd';
const Home = () =>
    <Space direction="vertical" size={16} style={{ marginTop: 20, width: '100%', justifyContent: 'center' }}>
        <Card title="Volume" >
            <VolumeCard wsPort={1234} />
        </Card>

    </Space>


export default Home