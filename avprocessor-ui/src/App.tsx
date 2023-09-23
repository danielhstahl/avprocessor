import React, { useEffect, useContext } from 'react';
import { Layout, Menu, Select, Space, Typography } from 'antd';
import { Outlet, useNavigate, useLocation, useRouteLoaderData } from "react-router-dom";
import { ROOT_ID } from './utils/constants'
import Speakers from './pages/Speakers'
import Advanced from './pages/Advanced'
import Home from './pages/Home'
import { SpeakerContext, Speaker } from './state/speaker'
import { FilterContext, Filter } from './state/filter'
import { Version, VersionContext } from './state/version'
import { getVersions } from './services/versions'
import { getConfiguration, ConfigPayload } from './services/configuration'
const { Header, Footer, Content } = Layout;
const { Text } = Typography;

type VersionConfigurationPayload = {
  versions: Version[],
  speakers: Speaker[],
  filters: Filter[],
  appliedVersion: string
}

export const deriveAppliedVersion = (versions: Version[]) => (versions.find(v => v.appliedVersion) || versions[versions.length - 1]).version
export const loader = () => {
  return getVersions().then(versions => {
    if (versions.length > 0) {
      const appliedVersion = deriveAppliedVersion(versions)
      return getConfiguration(appliedVersion).then(({ speakers, filters }) => ({ versions, speakers, filters, appliedVersion }))
    }
    else {
      return {
        versions
      }
    }
  })
}

export const SPEAKER_ROUTE = "/speakers"
export const ADVANCED_ROUTE = "/prompt"


export const MenuItems = [
  { key: "/", label: "Home", element: <Home /> },
  { key: SPEAKER_ROUTE, label: "Speaker Setup", element: <Speakers /> },
  { key: ADVANCED_ROUTE, label: "Advanced", element: <Advanced /> },
]

interface AppProps {
  getConfigurationProp?: (_: string) => Promise<ConfigPayload>
}


const App: React.FC<AppProps> = ({ getConfigurationProp = getConfiguration }: AppProps) => {
  const navigate = useNavigate()
  const location = useLocation()
  const { versions: fetchedVersions, speakers, filters, appliedVersion } = useRouteLoaderData(ROOT_ID) as VersionConfigurationPayload;
  const { setVersions, setSelectedVersion, selectedVersion, versions } = useContext(VersionContext)
  const { setSpeakers, setSpeakerBase, speakerConfiguration } = useContext(SpeakerContext)
  const { setFilters, setFilterBase } = useContext(FilterContext)
  useEffect(() => {
    setVersions(fetchedVersions)
  }, [fetchedVersions, setVersions])

  useEffect(() => {
    if (appliedVersion) {
      setSelectedVersion(appliedVersion)
    }
  }, [appliedVersion, setSelectedVersion]) //only called once on load

  useEffect(() => {
    if (speakers) {
      setSpeakers(speakers)
    }
  }, [speakers, setSpeakers])

  useEffect(() => {
    if (filters) {
      setFilters(filters)
    }
  }, [filters, setFilters])


  useEffect(() => {
    setSpeakerBase(speakerConfiguration)
    setFilterBase(speakerConfiguration)
  }, [speakerConfiguration, setSpeakerBase, setFilterBase])

  const onSelectVersion = (version: string) => {
    setSelectedVersion(version)
    getConfigurationProp(version).then(({ filters, speakers }) => {
      if (speakers && speakers.length > 0) {
        setSpeakers(speakers) //this will trigger a `setSpeakerBase` and `setFilterBase` since it will update the speakerConfiguration
        setFilters(filters)
      }
    })
  }

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
          <Select value={selectedVersion} onChange={onSelectVersion} options={versions.map(({ version }) => ({ value: version, label: version }))} style={{ width: '100%' }} />
        </Space>
        <Outlet />
      </Content>
      <Footer style={{ textAlign: 'center' }}>AV Processor</Footer>
    </Layout>
  );
};


export default App