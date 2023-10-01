import React, { useEffect } from 'react';
import { Layout, Menu } from 'antd';
import { Outlet, useNavigate, useLocation, useRouteLoaderData } from "react-router-dom";
import { ROOT_ID } from './utils/constants'
import Speakers from './pages/Speakers'
import Advanced from './pages/Advanced'
import Home from './pages/Home'
import { useSpeaker, Speaker, SpeakerAction } from './state/speaker'
import { useFilter, Filter, FilterAction } from './state/filter'
import { Version, VersionAction, useVersion } from './state/version'
import { getVersions } from './services/versions'
import { getConfiguration } from './services/configuration'
const { Header, Footer, Content } = Layout;

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
export const ADVANCED_ROUTE = "/advanced"


export const MenuItems = [
  { key: "/", label: "Home", element: <Home /> },
  { key: SPEAKER_ROUTE, label: "Speaker Setup", element: <Speakers /> },
  { key: ADVANCED_ROUTE, label: "Advanced", element: <Advanced /> },
]

const App: React.FC = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const { versions: fetchedVersions, speakers, filters, appliedVersion } = useRouteLoaderData(ROOT_ID) as VersionConfigurationPayload;
  const { state: { speakerConfiguration }, dispatch: speakerDispatch } = useSpeaker()
  const { dispatch: versionDispatch } = useVersion()
  const { dispatch: filterDispatch } = useFilter()

  useEffect(() => {
    versionDispatch({ type: VersionAction.INIT, value: fetchedVersions })
  }, [fetchedVersions, versionDispatch])

  useEffect(() => {
    if (appliedVersion) {
      versionDispatch({ type: VersionAction.SELECT, value: appliedVersion })
    }
  }, [appliedVersion, versionDispatch]) //only called once on load

  useEffect(() => {
    if (speakers) {
      speakerDispatch({ type: SpeakerAction.SET, value: speakers })
    }
  }, [speakers, speakerDispatch])

  useEffect(() => {
    if (filters) {
      filterDispatch({ type: FilterAction.SET, value: filters })
    }
  }, [filters, filterDispatch])

  useEffect(() => {
    speakerDispatch({ type: SpeakerAction.INIT, value: speakerConfiguration })
    filterDispatch({ type: FilterAction.INIT, value: speakerConfiguration })
  }, [speakerConfiguration, speakerDispatch, filterDispatch])

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
        <Outlet />
      </Content>
      <Footer style={{ textAlign: 'center' }}>AV Processor</Footer>
    </Layout>
  );
};


export default App