import { List, Space, Typography, InputNumber, Button, Row, Col } from 'antd';
import { DeleteOutlined, PlusOutlined } from '@ant-design/icons'
import { FilterWithIndex } from '../state/filter'
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import { constructVisualArray } from '../utils/peq'
import { Line } from 'react-chartjs-2';
import { ChartOptions } from "chart.js"
import {
    Chart as ChartJS,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
} from 'chart.js';
const { Text } = Typography
ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
);
const options: ChartOptions = {
    responsive: true,
    elements: {
        point: {
            radius: 0
        }
    },
    scales: {
        x: {
            ticks: {
                callback: function (value: string | number) {
                    return typeof value === "number" ? Number.parseInt(this.getLabelForValue(value)) : Number.parseInt(value);
                }
            }
        }
    },
    plugins: {
        legend: {
            display: false
        }

    },
};


type ActionProps = {
    filter: FilterWithIndex,
    updateFilter: (filter: FilterWithIndex) => void,
}

const FreqAction = ({ filter, updateFilter }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Frequency:</Text>
        <InputNumber
            value={filter.freq}
            onChange={v => v !== null && updateFilter({ ...filter, freq: v })}
            min={0}
            max={20000}
            {...intFormatter("hz")}
        />
    </Space>
}

const GainAction = ({ filter, updateFilter }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Gain:</Text>
        <InputNumber
            value={filter.gain}
            onChange={v => v !== null && updateFilter({ ...filter, gain: v })}
            step="0.5"
            min={-10}
            max={10}
            {...floatFormatter("db")}
        />
    </Space>
}

const QAction = ({ filter, updateFilter }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Q:</Text>
        <InputNumber
            value={filter.q}
            step="0.2"
            onChange={v => v !== null && updateFilter({ ...filter, q: v })}
            min={0}
            max={10}
        />
    </Space>
}


const PeqChartChartJS = ({ labels, values }: { labels: number[], values: number[] }) => {
    const jsdata = {
        labels,
        datasets: [
            {
                label: 'PEQ',
                data: values,
                borderColor: 'rgb(53, 162, 235)',
                backgroundColor: 'rgba(53, 162, 235, 0.5)',
            },
        ],
    };
    // @ts-ignore
    return <Line options={options} data={jsdata} />;

}

export type PeqProps = {
    filters: FilterWithIndex[],
    updateFilter: (_: FilterWithIndex) => void,
    removeFilter: (_: FilterWithIndex) => void,
    addFilter: () => void
}
const PeqRecord = ({ filters, updateFilter, removeFilter, addFilter }: PeqProps) => {
    const results = constructVisualArray(filters)
    return <Row style={{ minHeight: 100 }}>
        <Col md={24} lg={13} >
            <List
                itemLayout="horizontal"
                dataSource={filters}
                renderItem={(filter: FilterWithIndex) => (
                    <List.Item>
                        <FreqAction filter={filter} updateFilter={updateFilter} />
                        <GainAction filter={filter} updateFilter={updateFilter} />
                        <QAction filter={filter} updateFilter={updateFilter} />
                        <DeleteOutlined onClick={() => removeFilter(filter)} />
                    </List.Item>
                )}
                footer={<Button icon={<PlusOutlined />} onClick={addFilter}>Add Filter</Button>}
            />
        </Col>
        <Col xs={0} md={0} lg={11} style={{ paddingLeft: "10%" }}>
            <PeqChartChartJS labels={results.freq} values={results.freqResponse} />
        </Col>
    </Row>
}

export default PeqRecord