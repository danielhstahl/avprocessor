import React, { useEffect, useState, useContext } from 'react';
import { Layout, Menu, Select, Space, Typography } from 'antd';
import { Outlet, useNavigate, useLocation, useRouteLoaderData } from "react-router-dom";
import { ROOT_ID } from './utils/constants'
import Speakers from './pages/Speakers'
import Advanced from './pages/Advanced'
import Home from './pages/Home'
import { SpeakerContext } from './state/speaker'
import { FilterContext } from './state/filter'
const { Header, Footer, Content } = Layout;
const { Text } = Typography;

export const loader = () => fetch("/versions", {
  method: "GET",
})

export const SPEAKER_ROUTE = "/speakers"
export const PEQ_ROUTE = "/peq"
export const ADVANCED_ROUTE = "/prompt"


export const MenuItems = [
  { key: "/", label: "Home", element: <Home /> },
  { key: SPEAKER_ROUTE, label: "Speaker Setup", element: <Speakers /> },
  { key: ADVANCED_ROUTE, label: "Advanced", element: <Advanced /> },
]

type Version = {
  version: string
}


const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation()
  const versions = useRouteLoaderData(ROOT_ID) as Version[];
  const [version, setVersion] = useState(versions[versions.length - 1].version)
  const { setSpeakers, setSpeakerBase, speakerConfiguration } = useContext(SpeakerContext)
  const { setFilters, setFilterBase } = useContext(FilterContext)

  //todo, this is somewhat janky from a if then catch perspsective
  useEffect(() => {
    fetch(`/config/${version}`, {
      method: "GET",
    }).then(r => r.json()).then(({ speakers, filters }) => {
      if (speakers && speakers.length > 0) {
        setSpeakers(speakers)
        setFilters(filters)
      }
      else {
        setSpeakerBase(speakerConfiguration)
        setFilterBase(speakerConfiguration)
      }
    }).finally(() => {
      setSpeakerBase(speakerConfiguration)
      setFilterBase(speakerConfiguration)
    })
  }, [version])

  return (
    <Layout className="layout" style={{ minHeight: "100vh" }}>
      <Header style={{ display: 'flex', alignItems: 'center' }}>
        <div className="demo-logo" />
        <Menu
          theme="dark"
          mode="horizontal"
          onClick={({ key }) => navigate(key)}
          selectedKeys={[location.pathname]}
          items={MenuItems.map(({ key, label }) => ({ key, label }))}
        />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Space direction="horizontal" size="middle" style={{ display: 'flex' }}>
          <Text strong>Select Configuration Version</Text>
          <Select value={version} onChange={setVersion} options={versions.map(({ version }) => ({ value: version, label: version }))} style={{ width: '100%' }} />
        </Space>
        <Outlet />
      </Content>
      <Footer style={{ textAlign: 'center' }}>AV Processor</Footer>
    </Layout>
  );
};


export default App