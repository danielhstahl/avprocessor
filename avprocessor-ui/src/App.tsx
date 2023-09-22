import React, { useEffect, useContext } from 'react';
import { Layout, Menu, Select, Space, Typography } from 'antd';
import { Outlet, useNavigate, useLocation, useRouteLoaderData } from "react-router-dom";
import { ROOT_ID } from './utils/constants'
import Speakers from './pages/Speakers'
import Advanced from './pages/Advanced'
import Home from './pages/Home'
import { SpeakerContext } from './state/speaker'
import { FilterContext } from './state/filter'
import { Version, VersionContext } from './state/version'
const { Header, Footer, Content } = Layout;
const { Text } = Typography;

export const loader = () => fetch("/versions", {
  method: "GET",
})

export const SPEAKER_ROUTE = "/speakers"
export const ADVANCED_ROUTE = "/prompt"


export const MenuItems = [
  { key: "/", label: "Home", element: <Home /> },
  { key: SPEAKER_ROUTE, label: "Speaker Setup", element: <Speakers /> },
  { key: ADVANCED_ROUTE, label: "Advanced", element: <Advanced /> },
]

const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation()
  const fetchedVersions = useRouteLoaderData(ROOT_ID) as Version[];
  const { setVersions, setSelectedVersion, selectedVersion, versions } = useContext(VersionContext)
  useEffect(() => {
    setVersions(fetchedVersions)
    setSelectedVersion(fetchedVersions[fetchedVersions.length - 1].version)
  }, [fetchedVersions, setSelectedVersion, setVersions])

  const { setSpeakers, setSpeakerBase, speakerConfiguration } = useContext(SpeakerContext)
  const { setFilters, setFilterBase } = useContext(FilterContext)

  useEffect(() => {
    setSpeakerBase(speakerConfiguration)
    setFilterBase(speakerConfiguration)
  }, [speakerConfiguration, setSpeakerBase, setFilterBase])

  useEffect(() => {
    fetch(`/config/${selectedVersion}`, {
      method: "GET",
    }).then(r => r.json()).then(({ speakers, filters }) => {
      if (speakers && speakers.length > 0) {
        setSpeakers(speakers) //this will trigger a `setSpeakerBase` and `setFilterBase` since it will update the speakerConfiguration
        setFilters(filters)
      }
    })
  }, [selectedVersion, setSpeakers, setFilters])

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
          <Select value={selectedVersion} onChange={setSelectedVersion} options={versions.map(({ version }) => ({ value: version, label: version }))} style={{ width: '100%' }} />
        </Space>
        <Outlet />
      </Content>
      <Footer style={{ textAlign: 'center' }}>AV Processor</Footer>
    </Layout>
  );
};


export default App