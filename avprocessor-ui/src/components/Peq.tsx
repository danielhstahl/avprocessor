import { Typography, InputNumber, Button, Row, Col, Divider } from 'antd';
import { DeleteOutlined, PlusOutlined } from '@ant-design/icons'
import { FilterWithIndex } from '../state/filter'
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import { constructVisualArray } from '../utils/peq'
import { Line } from 'react-chartjs-2';
import { ChartOptions } from "chart.js"
import { inputStyle, textStyle, gutterStyle } from "./styles"
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
    return <Row align="middle" justify="center">
        <Col xs={0} md={12}>
            <Text style={textStyle} ellipsis={true}>Frequency:</Text>
        </Col>
        <Col xs={12} md={0}>
            <Text style={textStyle} ellipsis={true}>Freq:</Text>
        </Col>
        <Col xs={12}>
            <InputNumber
                style={inputStyle}
                value={filter.freq}
                onChange={v => v !== null && updateFilter({ ...filter, freq: v })}
                min={0}
                max={20000}
                {...intFormatter("hz")}
            />
        </Col>
    </Row>
}

const GainAction = ({ filter, updateFilter }: ActionProps) => {
    return <Row align="middle" justify="center">
        <Col xs={12}>
            <Text style={textStyle} ellipsis={true}>Gain:</Text>
        </Col>
        <Col xs={12}>
            <InputNumber
                style={inputStyle}
                value={filter.gain}
                onChange={v => v !== null && updateFilter({ ...filter, gain: v })}
                step="0.5"
                min={-10}
                max={10}
                {...floatFormatter("db")}
            />
        </Col>
    </Row>
}

const QAction = ({ filter, updateFilter }: ActionProps) => {
    return <Row align="middle" justify="center">
        <Col xs={12}>
            <Text style={textStyle} ellipsis={true}>Q:</Text>
        </Col>
        <Col xs={12}>
            <InputNumber
                style={inputStyle}
                value={filter.q}
                step="0.2"
                onChange={v => v !== null && updateFilter({ ...filter, q: v })}
                min={0}
                max={10}
            />
        </Col>
    </Row>
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
        <Col md={24} lg={15}>
            {filters.map(filter => {
                return <Row align="middle" justify="center">
                    <Col xs={18} md={24}>
                        <Row gutter={gutterStyle} style={{ marginBottom: 12 }} align="middle">
                            <Col xs={24} md={9}><FreqAction filter={filter} updateFilter={updateFilter} /></Col>
                            <Col xs={24} md={8}><GainAction filter={filter} updateFilter={updateFilter} /></Col>
                            <Col xs={24} md={6}><QAction filter={filter} updateFilter={updateFilter} /></Col>
                            <Col xs={0} md={1}><DeleteOutlined onClick={() => removeFilter(filter)} /></Col>
                        </Row>
                    </Col>
                    <Col xs={6} md={0} style={{ textAlign: "center" }}>
                        <DeleteOutlined onClick={() => removeFilter(filter)} />
                    </Col>
                    <Col xs={24} md={0}>
                        <Divider />
                    </Col>

                </Row>
            })}
            <Button icon={<PlusOutlined />} onClick={addFilter}>Add Filter</Button>
        </Col>
        <Col xs={0} md={0} lg={9} style={{ paddingLeft: "10%" }}>
            <PeqChartChartJS labels={results.freq} values={results.freqResponse} />
        </Col>
    </Row>
}

export default PeqRecord