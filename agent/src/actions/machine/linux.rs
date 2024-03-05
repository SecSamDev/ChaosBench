pub fn restart_host(_parameters : &TestParameters) -> ChaosResult<()> {
    sync();
    reboot(RB_AUTOBOOT);
}