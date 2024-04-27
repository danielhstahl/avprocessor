import React, { useEffect } from 'react';
import { Layout, Menu } from 'antd';
import { Outlet, useNavigate, useLocation, useRouteLoaderData } from "react-router-dom";
import { ROOT_ID } from './utils/constants'
import Speakers from './pages/Speakers'
import Advanced from './pages/Advanced'
import Home from './pages/Home'
import { useSpeaker, SpeakerAction } from './state/speaker'
import { useFilter, FilterAction } from './state/filter'
import { Version, VersionAction, useVersion } from './state/version'
import { getVersions } from './services/versions'
import { ConfigPayload, getConfiguration } from './services/configuration'
import { DelayAction, useDelay } from './state/delay';
import { DeviceAction, useDevice } from './state/device';
const { Header, Footer, Content } = Layout;

interface VersionConfigurationPayload extends ConfigPayload {
  versions: Version[],
  appliedVersion: number
}

export const deriveAppliedVersion = (versions: Version[]) => (versions.find(v => v.appliedVersion) || versions[versions.length - 1]).version
export const loader = () => {
  return getVersions().then(versions => {
    if (versions.length > 0) {
      const appliedVersion = deriveAppliedVersion(versions)
      return getConfiguration(appliedVersion).then(({ speakers, filters, selectedDistance }) => ({ versions, speakers, filters, appliedVersion, selectedDistance }))
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
  const { versions: fetchedVersions, speakers, filters, appliedVersion, selectedDistance, device } = useRouteLoaderData(ROOT_ID) as VersionConfigurationPayload;
  const { state: { speakerConfiguration }, dispatch: speakerDispatch } = useSpeaker()
  const { dispatch: versionDispatch } = useVersion()
  const { dispatch: filterDispatch } = useFilter()
  const { dispatch: delayDispatch } = useDelay()
  const { dispatch: deviceDispatch } = useDevice()

  useEffect(() => {
    versionDispatch({ type: VersionAction.INIT, value: fetchedVersions })
  }, [fetchedVersions, versionDispatch])

  useEffect(() => {
    if (appliedVersion) {
      versionDispatch({ type: VersionAction.SELECT, value: appliedVersion })

    }
  }, [appliedVersion, versionDispatch]) //only called once on load

  useEffect(() => {
    selectedDistance && delayDispatch({
      type: DelayAction.UPDATE, value: selectedDistance
    })
  }, [selectedDistance, delayDispatch])

  useEffect(() => {
    device && deviceDispatch({
      type: DeviceAction.UPDATE,
      value: device
    })
  }, [device, deviceDispatch])


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